package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
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
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallModes : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val installModesSchema = get<SchemasImpl>().installerSchema.definitions.installModes

    fun Terminal.installModesPrompt() {
        do {
            println(brightYellow(installModesInfo))
            println(cyan(installModesExample))
            val input = prompt(
                prompt = brightWhite(PromptType.InstallModes.toString()),
                default = getPreviousValue()?.joinToString(", ")?.also {
                    println(gray("Previous install modes: $it"))
                }
            )?.trim()?.convertToYamlList(installModesSchema.uniqueItems)?.toInstallModes()
            val (installModesValid, error) = areInstallModesValid(input)
            if (installModesValid == Validation.Success && input != null) {
                installerManifestData.installModes = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (installModesValid != Validation.Success)
    }

    private fun areInstallModesValid(
        installModes: Iterable<InstallerManifest.InstallModes>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installModesSchema = installerSchema.definitions.installModes
        return when {
            (installModes?.count() ?: 0) > installModesSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = installModesSchema.maxItems)
            }
            installModes?.any { it !in InstallerManifest.InstallModes.values() } == true -> {
                Validation.InvalidInstallMode to Errors.invalidEnum(
                    Validation.InvalidInstallMode,
                    InstallerManifest.InstallModes.values().map { it.toString() }
                )
            }
            else -> Validation.Success to null
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
}
