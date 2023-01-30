package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.InstallerSchema

object Commands : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val commandsSchema = get<SchemasImpl>().installerSchema.definitions.commands

    fun Terminal.commandsPrompt() {
        println(
            colors.brightYellow(
                "${Prompts.optional} ${commandsSchema.description} (Max ${commandsSchema.maxItems})"
            )
        )
        installerManifestData.commands = prompt(
            prompt = const,
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous commands: $it") },
            convert = { input ->
                areCommandsValid(input.convertToYamlList())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList()
        println()
    }

    private fun areCommandsValid(
        commands: Iterable<String>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): String? {
        val commandsSchema = installerSchema.definitions.commands
        return when {
            (commands?.count() ?: 0) > commandsSchema.maxItems -> {
                Errors.invalidLength(max = commandsSchema.maxItems)
            }
            commands?.any { it.length > commandsSchema.items.maxLength } == true -> {
                Errors.invalidLength(
                    min = commandsSchema.items.minLength,
                    max = commandsSchema.items.maxLength,
                    items = commands.filter { it.length > commandsSchema.items.maxLength }
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

    private const val const = "Commands"
}
