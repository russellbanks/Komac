package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object Commands : KoinComponent {
    fun Terminal.commandsPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val commandsSchema = schemasImpl.installerSchema.definitions.commands
        do {
            println(brightYellow("${Prompts.optional} ${commandsSchema.description} (Max ${commandsSchema.maxItems})"))
            val input = prompt(brightWhite(PromptType.Commands.toString()))
                ?.trim()?.convertToYamlList(commandsSchema.uniqueItems)
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
}
