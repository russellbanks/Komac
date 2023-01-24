package data.installer

import Errors
import ExitCode
import Validation
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
import kotlin.system.exitProcess

object InstallModes : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val installModesSchema = get<SchemasImpl>().installerSchema.definitions.installModes

    fun Terminal.installModesPrompt() {
        println(colors.brightYellow(installModesInfo))
        info(installModesExample)
        installerManifestData.installModes = prompt(
            prompt = const,
            default = getPreviousValue()?.joinToString(", ")?.also { muted("Previous install modes: $it") },
            convert = { input ->
                areInstallModesValid(input.convertToYamlList(installModesSchema.uniqueItems)?.toInstallModes())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(installModesSchema.uniqueItems)?.toInstallModes() ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun areInstallModesValid(installModes: Iterable<InstallerManifest.InstallModes>?): String? {
        return when {
            (installModes?.count() ?: 0) > installModesSchema.maxItems -> {
                Errors.invalidLength(max = installModesSchema.maxItems)
            }
            installModes?.any { it !in InstallerManifest.InstallModes.values() } == true -> {
                Errors.invalidEnum(
                    Validation.InvalidInstallMode,
                    InstallerManifest.InstallModes.values().map { it.toString() }
                )
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<Enum<*>>? {
        return previousManifestData.remoteInstallerData?.let {
            it.installModes ?: it.installers[installerManifestData.installers.size].installModes
        }
    }

    private val installModesInfo = "${Prompts.optional} ${installModesSchema.description}"
    private val installModesExample = "Options: ${InstallerManifest.InstallModes.values().joinToString(", ")}"

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes>? {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }.ifEmpty { null }
    }

    private const val const = "Install Modes"
}
