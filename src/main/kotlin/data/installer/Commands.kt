package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object Commands : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val commandsSchema = get<SchemasImpl>().installerSchema.definitions.commands
    suspend fun Terminal.commandsPrompt() {
        do {
            println(brightYellow("${Prompts.optional} ${commandsSchema.description} (Max ${commandsSchema.maxItems})"))
            val input = prompt(
                prompt = brightWhite(PromptType.Commands.toString()),
                default = getPreviousValue()?.joinToString(", ")?.also {
                    println(gray("Previous commands: $it"))
                }
            )?.trim()?.convertToYamlList(commandsSchema.uniqueItems)
            val (commandsValid, error) = areCommandsValid(input)
            if (commandsValid == Validation.Success) installerManifestData.commands = input
            error?.let { println(red(it)) }
            println()
        } while (commandsValid != Validation.Success)
    }

    fun areCommandsValid(
        commands: Iterable<String>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val commandsSchema = installerSchema.definitions.commands
        return when {
            (commands?.count() ?: 0) > commandsSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = commandsSchema.maxItems)
            }
            commands?.any { it.length > commandsSchema.items.maxLength } == true -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = commandsSchema.items.minLength,
                    max = commandsSchema.items.maxLength,
                    items = commands.filter { it.length > commandsSchema.items.maxLength }
                )
            }
            else -> Validation.Success to null
        }
    }

    private suspend fun getPreviousValue(): List<String>? {
        return sharedManifestData.remoteInstallerData.await().let {
            it?.commands ?: it?.installers?.get(installerManifestData.installers.size)?.commands
        }
    }
}
