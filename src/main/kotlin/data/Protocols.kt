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
        val uniqueItems = installerSchemaImpl.installerSchema.definitions.protocols.uniqueItems
        do {
            println(brightYellow(Prompts.protocolsInfo(installerSchemaImpl.installerSchema)))
            val input = prompt(brightWhite(PromptType.Protocols.toString()))?.trim()?.convertToYamlList(uniqueItems)
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
        val protocolsMaxItems = installerSchema.definitions.protocols.maxItems
        val protocolMaxLength = installerSchema.definitions.protocols.items.maxLength
        return when {
            (protocols?.count() ?: 0) > protocolsMaxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = protocolsMaxItems)
            }
            protocols?.any { it.length > protocolMaxLength } == true -> {
                Validation.InvalidLength to Errors.invalidLength(max = protocolMaxLength)
            }
            else -> Validation.Success to null
        }
    }
}
