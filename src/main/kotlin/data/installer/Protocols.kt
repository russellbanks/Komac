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
import schemas.manifest.InstallerManifest

object Protocols : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val protocolsSchema = get<SchemasImpl>().installerSchema.definitions.protocols

    fun Terminal.protocolsPrompt() {
        println(
            colors.brightYellow("${Prompts.optional} ${protocolsSchema.description} (Max ${protocolsSchema.maxItems})")
        )
        installerManifestData.protocols = prompt(
            prompt = InstallerManifest::protocols.name.replaceFirstChar { it.titlecase() },
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous protocols: $it") },
            convert = {
                val error = areProtocolsValid(it.trim().convertToYamlList(protocolsSchema.uniqueItems))
                if (error != null) {
                    ConversionResult.Invalid(error)
                } else {
                    ConversionResult.Valid(it.trim())
                }
            }
        )?.convertToYamlList(protocolsSchema.uniqueItems)
        println()
    }

    private fun areProtocolsValid(protocols: Iterable<String>?): String? {
        return when {
            (protocols?.count() ?: 0) > protocolsSchema.maxItems -> Errors.invalidLength(max = protocolsSchema.maxItems)
            protocols?.any { it.length > protocolsSchema.items.maxLength } == true -> {
                Errors.invalidLength(
                    max = protocolsSchema.items.maxLength,
                    items = protocols.filter { it.length > protocolsSchema.items.maxLength }
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.protocols ?: it.installers.getOrNull(installerManifestData.installers.size)?.protocols
        }
    }
}
