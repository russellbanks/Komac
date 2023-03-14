package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import detection.files.msi.Msi

class PackageName(
    private val msi: Msi?,
    private val previousPackageName: String?
) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightGreen(nameInfo))
        info(example)
        msi?.productName?.let { info("Detected from MSI: $it") }
        return prompt(
            prompt = const,
            default = previousPackageName?.also { muted("Previous package name: $it") }
        ) { input ->
            getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length < minLength || input.length > maxLength -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    companion object {
        private const val const = "Package Name"
        private const val nameInfo = "Enter the package name"
        private const val example = "Example: Microsoft Teams"
        private const val minLength = 2
        private const val maxLength = 256
    }
}
