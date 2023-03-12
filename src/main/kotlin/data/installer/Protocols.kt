package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.InstallerManifest

class Protocols(
    private val previousInstallerManifest: InstallerManifest?,
    private val installersSize: Int
) : CommandPrompt<List<String>> {
    override fun prompt(terminal: Terminal): List<String>? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = InstallerManifest::protocols.name.replaceFirstChar(Char::titlecase),
            default = getPreviousValue()?.also { muted("Previous protocols: $it") }
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
            convertedInput.any { it.length > maxLength } -> {
                Errors.invalidLength(max = maxLength, items = convertedInput.filter { it.length > maxLength })
            }

            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousInstallerManifest?.let { it.protocols ?: it.installers.getOrNull(installersSize)?.protocols }
    }

    companion object {
        private const val maxItems = 64
        private const val maxLength = 2048
        private const val uniqueItems = true
        private const val description = "List of protocols the package provides a handler for"
    }
}
