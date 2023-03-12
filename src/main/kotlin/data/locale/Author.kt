package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import schemas.manifest.DefaultLocaleManifest

class Author(private val previousAuthor: String?) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightYellow(authorInfo))
        return prompt(
            prompt = DefaultLocaleManifest::author.name.replaceFirstChar(Char::titlecase),
            default = previousAuthor?.also { muted("Previous author: $it") }
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
        private const val authorInfo = "${Prompts.optional} Enter the package author"
        private const val minLength = 2
        private const val maxLength = 256
    }
}
