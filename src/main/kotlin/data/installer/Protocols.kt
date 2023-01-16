package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.InstallerSchema

object Protocols : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val protocolsSchema = get<SchemasImpl>().installerSchema.definitions.protocols

    fun Terminal.protocolsPrompt() {
        do {
            println(
                brightYellow("${Prompts.optional} ${protocolsSchema.description} (Max ${protocolsSchema.maxItems})")
            )
            val input = prompt(
                prompt = brightWhite(PromptType.Protocols.toString()),
                default = getPreviousValue()?.joinToString(", ")?.also { println(gray("Previous protocols: $it")) }
            )?.trim()?.convertToYamlList(protocolsSchema.uniqueItems)
            val (protocolsValid, error) = areProtocolsValid(input)
            if (protocolsValid == Validation.Success) installerManifestData.protocols = input
            error?.let { println(brightRed(it)) }
            println()
        } while (protocolsValid != Validation.Success)
    }

    private fun areProtocolsValid(
        protocols: Iterable<String>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
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

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.protocols ?: it.installers[installerManifestData.installers.size].protocols
        }
    }
}
