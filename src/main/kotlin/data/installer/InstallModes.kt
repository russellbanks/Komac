package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.SharedManifestData
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
    private val sharedManifestData: SharedManifestData by inject()
    private val installModesSchema = get<SchemasImpl>().installerSchema.definitions.installModes

    suspend fun Terminal.installModesPrompt() {
        do {
            println(brightYellow(installModesInfo))
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
            error?.let { println(red(it)) }
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

    private suspend fun getPreviousValue(): List<Enum<*>>? {
        return sharedManifestData.remoteInstallerData.await().let {
            it?.installModes ?: it?.installers?.get(installerManifestData.installers.size)?.installModes
        }
    }

    private val installModesInfo = buildString {
        append(Prompts.optional)
        append(" ")
        append(installModesSchema.description)
        append(". Options: ")
        append(InstallerManifest.InstallModes.values().joinToString(", "))
    }

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes>? {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }.ifEmpty { null }
    }
}
