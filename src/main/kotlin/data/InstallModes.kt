package data

import Errors
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
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl

object InstallModes : KoinComponent {
    fun Terminal.installModesPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl by inject()
        val installModesSchema = installerSchemaImpl.installerSchema.definitions.installModes
        do {
            println(brightYellow(promptInfo(installModesSchema)))
            val input = prompt(brightWhite(PromptType.InstallModes.toString()))
                ?.trim()?.convertToYamlList(installModesSchema.uniqueItems)
            val (installModesValid, error) = areInstallModesValid(input)
            if (installModesValid == Validation.Success && input != null) {
                installerManifestData.installModes = input.toInstallModes()
            }
            error?.let { println(red(it)) }
            println()
        } while (installModesValid != Validation.Success)
    }

    private fun areInstallModesValid(
        installModes: Iterable<String>?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installModesSchema = installerSchema.definitions.installModes
        return when {
            (installModes?.count() ?: 0) > installModesSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = installModesSchema.maxItems)
            }
            installModes?.any { it !in installModesSchema.items.enum } == true -> {
                Validation.InvalidInstallMode to Errors.invalidEnum(
                    Validation.InvalidInstallMode,
                    installModesSchema.items.enum
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun promptInfo(installModesDefinitions: InstallerSchema.Definitions.InstallModes): String {
        return buildString {
            append(Prompts.optional)
            append(" ")
            append(installModesDefinitions.description)
            append(". Options: ")
            append(installModesDefinitions.items.enum.joinToString(", "))
        }
    }

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes>? {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }.ifEmpty { null }
    }
}
