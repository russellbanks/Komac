package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerType : KoinComponent {
    fun Terminal.installerTypePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl = get()
        val installerTypeSchema = schemasImpl.installerSchema.definitions.installerType
        do {
            println(brightGreen(installerTypeInfo(installerTypeSchema)))
            val input = prompt(brightWhite(PromptType.InstallerType.toString()))?.trim()?.lowercase()
            val (installerTypeValid, error) = isInstallerTypeValid(input, installerTypeSchema)
            error?.let { println(red(it)) }
            if (installerTypeValid == Validation.Success && input != null) {
                installerManifestData.installerType = input.toInstallerType()
            }
            println()
        } while (installerTypeValid != Validation.Success)
    }

    fun isInstallerTypeValid(
        installerType: String?,
        installerTypeSchema: InstallerSchema.Definitions.InstallerType
    ): Pair<Validation, String?> {
        return when {
            installerType.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerType)
            !installerTypeSchema.enum.contains(installerType) -> {
                Validation.InvalidInstallerType to Errors.invalidEnum(
                    Validation.InvalidInstallerType,
                    installerTypeSchema.enum
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun String.toInstallerType(): InstallerManifest.InstallerType {
        InstallerManifest.InstallerType.values().forEach {
            if (it.toString().lowercase() == this) return it
        }
        throw IllegalArgumentException("Invalid installer type: $this")
    }

    private fun installerTypeInfo(installerTypeSchema: InstallerSchema.Definitions.InstallerType): String {
        return buildString {
            append("${Prompts.required} Enter the installer type. Options: ")
            append(installerTypeSchema.enum.joinToString(", "))
        }
    }
}
