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

object InstallModes : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.installModesPrompt() {
        println(colors.brightYellow(installModesInfo))
        info(installModesExample)
        installerManifestData.installModes = prompt(
            prompt = const,
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous install modes: $it") },
            convert = { input ->
                areInstallModesValid(input.convertToYamlList(uniqueItems).toInstallModes())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(uniqueItems)?.toInstallModes()
        println()
    }

    private fun areInstallModesValid(installModes: Iterable<InstallerManifest.InstallModes>?): String? {
        return when {
            (installModes?.count() ?: 0) > maxItems -> Errors.invalidLength(max = maxItems)
            installModes?.any { it !in InstallerManifest.InstallModes.values() } == true -> {
                Errors.invalidEnum(
                    InstallerManifest.InstallModes.values().map { it.toString() }
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<Enum<*>>? {
        return previousManifestData.remoteInstallerData?.let {
            it.installModes ?: it.installers.getOrNull(installerManifestData.installers.size)?.installModes
        }
    }

    private const val installModesInfo = "${Prompts.optional} Enter the ist of supported installer modes"
    private val installModesExample = "Options: ${InstallerManifest.InstallModes.values().joinToString(", ")}"

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes>? {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }.ifEmpty { null }
    }

    private const val const = "Install Modes"
    private const val maxItems = 3
    private const val uniqueItems = true
}
