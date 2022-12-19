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
import schemas.InstallerSchemaImpl

object Architecture : KoinComponent {

    fun Terminal.architecturePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl by inject()
        do {
            println(brightGreen(Prompts.architectureInfo(installerSchemaImpl.installerSchema)))
            val input = prompt(brightWhite(PromptType.Architecture.toString()))?.trim()?.lowercase()
            val (architectureValid, error) = isArchitectureValid(input)
            error?.let { println(red(it)) }
            if (architectureValid == Validation.Success && input != null) {
                installerManifestData.architecture = input.toArchitecture()
            }
            println()
        } while (architectureValid != Validation.Success)
    }

    fun isArchitectureValid(
        architecture: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val architecturesEnum = Enum.architecture(installerSchema)
        return when {
            architecture.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Architecture)
            !architecturesEnum.contains(architecture) -> {
                Validation.InvalidArchitecture to Errors.invalidEnum(Validation.InvalidArchitecture, architecturesEnum)
            }
            else -> Validation.Success to null
        }
    }

    private fun String.toArchitecture(): InstallerManifest.Installer.Architecture {
        InstallerManifest.Installer.Architecture.values().forEach {
            if (it.toString().lowercase() == this) return it
        }
        throw IllegalArgumentException("Invalid architecture: $this")
    }
}
