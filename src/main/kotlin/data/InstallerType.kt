package data

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
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl

object InstallerType : KoinComponent {
    fun Terminal.installerTypePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl = get()
        do {
            println(brightGreen(Prompts.installerTypeInfo(installerSchemaImpl.installerSchema)))
            installerManifestData.installerType = prompt(brightWhite(Prompts.installerType))?.trim()?.lowercase()
            val (installerTypeValid, error) = isInstallerTypeValid(installerManifestData.installerType)
            error?.let { println(red(it)) }
            println()
        } while (installerTypeValid != Validation.Success)
    }

    fun isInstallerTypeValid(
        installerType: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
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
}
