package data.installer

import Errors
import com.github.ajalt.clikt.core.CliktError
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
            prompt = colors.brightWhite(const),
            default = getPreviousValue()?.also { muted("Previous commands: $it") },
            convert = {
                val inputAsList = it.convertToYamlList(commandsSchema.uniqueItems)
                val error = areCommandsValid(inputAsList)
                if (error != null) {
                    ConversionResult.Invalid(error.message!!)
                } else {
                    ConversionResult.Valid(inputAsList)
                }
            }
        )
        println()
    }

    private fun areCommandsValid(
        commands: Iterable<String>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): CliktError? {
        val commandsSchema = installerSchema.definitions.commands
        return when {
            (commands?.count() ?: 0) > commandsSchema.maxItems -> {
                CliktError(Errors.invalidLength(max = commandsSchema.maxItems))
            }
            commands?.any { it.length > commandsSchema.items.maxLength } == true -> {
                CliktError(
                    Errors.invalidLength(
                        min = commandsSchema.items.minLength,
                        max = commandsSchema.items.maxLength,
                        items = commands.filter { it.length > commandsSchema.items.maxLength }
                    )
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.commands ?: it.installers[installerManifestData.installers.size].commands
        }
    }

    private const val const = "Commands"
}
