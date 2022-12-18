package data

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
import schemas.InstallerSchemaImpl

object Protocols : KoinComponent {
    fun Terminal.protocolsPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl by inject()
        val protocolsSchema = installerSchemaImpl.installerSchema.definitions.protocols
        do {
            println(
                brightYellow("${Prompts.optional} ${protocolsSchema.description} (Max ${protocolsSchema.maxItems})")
            )
            val input = prompt(brightWhite(PromptType.Protocols.toString()))
                ?.trim()?.convertToYamlList(protocolsSchema.uniqueItems)
            val (protocolsValid, error) = areProtocolsValid(input)
            if (protocolsValid == Validation.Success) installerManifestData.protocols = input
            error?.let { println(red(it)) }
            println()
        } while (protocolsValid != Validation.Success)
    }

    fun areProtocolsValid(
        protocols: Iterable<String>?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val protocolsSchema = installerSchema.definitions.protocols
        return when {
            (protocols?.count() ?: 0) > protocolsSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = protocolsSchema.maxItems)
            }
            protocols?.any { it.length > protocolsSchema.items.maxLength } == true -> {
                Validation.InvalidLength to Errors.invalidLength(
                    max = protocolsSchema.items.maxLength,
                    items = protocols.filter { it.length > protocolsSchema.items.maxLength }
                )
            }
            else -> Validation.Success to null
        }
    }
}
