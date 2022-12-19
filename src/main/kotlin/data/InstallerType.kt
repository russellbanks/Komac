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
import schemas.Enum
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerType : KoinComponent {
    fun Terminal.installerTypePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl = get()
        do {
            println(brightGreen(Prompts.installerTypeInfo(schemasImpl.installerSchema)))
            val input = prompt(brightWhite(PromptType.InstallerType.toString()))?.trim()?.lowercase()
            val (installerTypeValid, error) = isInstallerTypeValid(input)
            error?.let { println(red(it)) }
            if (installerTypeValid == Validation.Success && input != null) {
                installerManifestData.installerType = input.toInstallerType()
            }
            println()
        } while (installerTypeValid != Validation.Success)
    }

    fun isInstallerTypeValid(
        installerType: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerTypesEnum = Enum.installerType(installerSchema)
        return when {
            installerType.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerType)
            !installerTypesEnum.contains(installerType) -> {
                Validation.InvalidInstallerType to Errors.invalidEnum(
                    Validation.InvalidInstallerType,
                    installerTypesEnum
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
}
