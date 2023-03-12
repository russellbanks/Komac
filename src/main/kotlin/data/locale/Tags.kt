package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import detection.github.GitHubDetection
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.DefaultLocaleManifest

class Tags(
    private val gitHubDetection: GitHubDetection?,
    private val previousTags: List<String>?
) : CommandPrompt<List<String>> {
    override fun prompt(terminal: Terminal): List<String>? = with(terminal) {
        return gitHubDetection?.topics ?: let {
            println(colors.brightYellow(tagsInfo))
            info(example)
            prompt(
                prompt = DefaultLocaleManifest::tags.name.replaceFirstChar(Char::titlecase),
                default = previousTags?.also { muted("Previous tags: $it") }
            ) { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().convertToList(uniqueItems))
            }
        }
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToList(uniqueItems)
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxCount -> Errors.invalidLength(max = maxCount)
            convertedInput.any { it.length > maxLength } -> {
                Errors.invalidLength(
                    min = minLength,
                    max = maxLength,
                    items = convertedInput.filter { it.length > maxLength }
                )
            }
            else -> null
        }
    }

    private val tagsInfo = buildString {
        append(Prompts.optional)
        append(" Enter any tags that would be useful to discover this tool. ")
        append("(Max $maxCount)")
    }

    companion object {
        private const val example = "Example: zip, c++, photos, OBS"
        private const val maxCount = 16
        private const val maxLength = 40
        private const val minLength = 1
        private const val uniqueItems = true
    }
}
