package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object InstallModes : KoinComponent, CommandPrompt<List<InstallerManifest.InstallModes>> {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<InstallerManifest.InstallModes> = with(terminal) {
        println(colors.brightYellow(installModesInfo))
        info(installModesExample)
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous install modes: $it") },
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().convertToYamlList(uniqueItems).toInstallModes())
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToYamlList(uniqueItems)?.toInstallModes()
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxItems -> Errors.invalidLength(max = maxItems)
            convertedInput.any { it !in InstallerManifest.InstallModes.values() } -> {
                Errors.invalidEnum(InstallerManifest.InstallModes.values().map { it.toString() })
            }
            else -> null
        }
    }

    private suspend fun getPreviousValue(): List<InstallerManifest.InstallModes>? = with(allManifestData) {
        return previousManifestData.remoteInstallerData.await()?.let { installerManifest ->
            installerManifest.installModes
                ?: installerManifest.installers
                    .getOrNull(installers.size)
                    ?.installModes
                    ?.map { it.toManifestInstallMode() }
        }
    }

    private const val installModesInfo = "${Prompts.optional} Enter the list of supported installer modes"
    private val installModesExample = "Options: ${InstallerManifest.InstallModes.values().joinToString(", ")}"

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes> {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }
    }

    private const val const = "Install Modes"
    private const val maxItems = 3
    private const val uniqueItems = true
}
