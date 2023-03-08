package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import input.Prompts

class Description {
    class Short(
        private val allManifestData: AllManifestData,
        private val previousShortDescription: String?
    ) : CommandPrompt<String?> {
        override fun prompt(terminal: Terminal): String? = with(terminal) {
            if (allManifestData.gitHubDetection?.shortDescription != null && previousShortDescription != null) {
                return allManifestData.gitHubDetection?.shortDescription
            }
            println(colors.brightGreen("${Prompts.required} Enter the short package description"))
            allManifestData.msix?.description?.let { info("Description from installer: $it") }
            return prompt(
                prompt = "Short Description",
                default = previousShortDescription?.also { muted("Previous short description: $it") }
            ) { input ->
                getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        }

        override fun getError(input: String?): String? {
            return when {
                input == null -> null
                input.isBlank() -> Errors.blankInput("Short description")
                input.length < minLength || input.length > maxLength -> {
                    Errors.invalidLength(min = minLength, max = maxLength)
                }
                else -> null
            }
        }

        companion object {
            private const val minLength = 3
            private const val maxLength = 256
        }
    }

    class Long(private val previousDescription: String?) : CommandPrompt<String?> {
        override fun prompt(terminal: Terminal): String? = with(terminal) {
            println(colors.brightGreen("${Prompts.required} Enter the full package description"))
            return prompt(
                prompt = "Description",
                default = previousDescription?.also { muted("Previous description: $it") }
            ) { input ->
                getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        }

        override fun getError(input: String?): String? {
            return when {
                input == null -> null
                input.isBlank() -> Errors.blankInput("Description")
                input.length < minLength || input.length > maxLength -> {
                    Errors.invalidLength(min = minLength, max = maxLength)
                }
                else -> null
            }
        }

        companion object {
            private const val minLength = 3
            private const val maxLength = 10000
        }
    }
}
