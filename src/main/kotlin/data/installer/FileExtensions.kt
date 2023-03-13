package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.InstallerManifest

class FileExtensions(
    private val previousInstallerManifest: InstallerManifest?,
    private val installersSize: Int
) : CommandPrompt<List<String>> {
    override fun prompt(terminal: Terminal): List<String>? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous file extensions: $it") }
        ) { input ->
            getError(input)
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(input.trim().convertToList(uniqueItems))
        }
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToList(uniqueItems)
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxItems -> Errors.invalidLength(max = maxItems)
            convertedInput.any { !it.matches(regex) } -> {
                Errors.invalidRegex(regex = regex, items = convertedInput.filterNot { it matches regex })
            }
            convertedInput.any { it.length > maxItemLength } -> {
                Errors.invalidLength(max = maxItemLength, items = convertedInput.filter { it.length > maxItemLength })
            }

            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousInstallerManifest?.let {
            it.fileExtensions ?: it.installers.getOrNull(installersSize)?.fileExtensions
        }
    }

    companion object {
        private const val const = "File Extensions"
        private const val description = "List of file extensions the package could support"
        private const val maxItems = 512
        private const val maxItemLength = 64
        private const val pattern = "^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"
        private const val uniqueItems = true
        private val regex = Regex(pattern)
    }
}
