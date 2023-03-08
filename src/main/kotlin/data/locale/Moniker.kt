package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import schemas.manifest.DefaultLocaleManifest

class Moniker(private val previousMoniker: String?) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightYellow(monikerInfo))
        info(monikerExample)
        return prompt(
            prompt = DefaultLocaleManifest::moniker.name.replaceFirstChar(Char::titlecase),
            default = previousMoniker?.also { muted("Previous moniker: $it") }
        ) { input ->
            getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isNotBlank() && (input.length < minLength || input.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    companion object {
        private const val monikerInfo = "${Prompts.optional} Enter the Moniker (friendly name/alias)."
        private const val monikerExample = "Example: vscode"
        private const val minLength = 1
        private const val maxLength = 40
    }
}
