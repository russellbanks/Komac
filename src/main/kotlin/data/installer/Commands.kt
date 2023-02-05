package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest

object Commands : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.commandsPrompt() {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        installerManifestData.commands = prompt(
            prompt = InstallerManifest::commands.name.replaceFirstChar { it.titlecase() },
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous commands: $it") },
            convert = { input ->
                areCommandsValid(input.convertToYamlList())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(uniqueItems)
        println()
    }

    private fun areCommandsValid(commands: Iterable<String>): String? {
        return when {
            commands.count() > maxItems -> Errors.invalidLength(max = maxItems)
            commands.any { it.length > maxItemLength } -> {
                Errors.invalidLength(
                    min = minItemLength,
                    max = maxItemLength,
                    items = commands.filter { it.length > maxItemLength }
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.commands ?: it.installers.getOrNull(installerManifestData.installers.size)?.commands
        }
    }

    private const val description = "List of commands or aliases to run the package"
    private const val maxItems = 16
    private const val minItemLength = 1
    private const val maxItemLength = 40
    private const val uniqueItems = true
}
