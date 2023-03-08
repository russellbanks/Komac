package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.InstallerManifest

class Commands(
    private val previousInstallerManifest: InstallerManifest?,
    private val installersSize: Int
) : CommandPrompt<List<String>> {
    override fun prompt(terminal: Terminal): List<String>? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = InstallerManifest::commands.name.replaceFirstChar(Char::titlecase),
            default = getPreviousValue()?.also { muted("Previous commands: $it") }
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
            convertedInput.any { it.length > maxItemLength } -> {
                Errors.invalidLength(
                    min = minItemLength,
                    max = maxItemLength,
                    items = convertedInput.filter { it.length > maxItemLength }
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousInstallerManifest?.let { it.commands ?: it.installers.getOrNull(installersSize)?.commands }
    }

    companion object {
        private const val description = "List of commands or aliases to run the package"
        private const val maxItems = 16
        private const val minItemLength = 1
        private const val maxItemLength = 40
        private const val uniqueItems = true
    }
}
