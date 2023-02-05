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

object Protocols : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.protocolsPrompt() {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        installerManifestData.protocols = prompt(
            prompt = InstallerManifest::protocols.name.replaceFirstChar { it.titlecase() },
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous protocols: $it") },
            convert = { input ->
                areProtocolsValid(input.trim().convertToYamlList(uniqueItems))
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(uniqueItems)
        println()
    }

    private fun areProtocolsValid(protocols: Iterable<String>?): String? {
        return when {
            (protocols?.count() ?: 0) > maxItems -> Errors.invalidLength(max = maxItems)
            protocols?.any { it.length > maxLength } == true -> {
                Errors.invalidLength(max = maxLength, items = protocols.filter { it.length > maxLength })
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.protocols ?: it.installers.getOrNull(installerManifestData.installers.size)?.protocols
        }
    }

    private const val maxItems = 64
    private const val maxLength = 2048
    private const val uniqueItems = true
    private const val description = "List of protocols the package provides a handler for"
}
