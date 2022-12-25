package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object Architecture : KoinComponent {

    suspend fun Terminal.architecturePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val sharedManifestData: SharedManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val architectureSchema = schemasImpl.installerSchema.definitions.architecture
        do {
            println(brightGreen(architectureInfo(architectureSchema)))
            val input = prompt(
                prompt = brightWhite(PromptType.Architecture.toString()),
                default = sharedManifestData.remoteInstallerData.await()?.installers
                    ?.get(installerManifestData.installers.size)?.architecture?.also {
                        println(gray("Previous value: $it"))
                    }?.toString()
            )?.trim()?.lowercase()
            val (architectureValid, error) = isArchitectureValid(input, architectureSchema)
            error?.let { println(red(it)) }
            if (architectureValid == Validation.Success && input != null) {
                installerManifestData.architecture = input.toArchitecture()
            }
            println()
        } while (architectureValid != Validation.Success)
    }

    fun isArchitectureValid(
        architecture: String?,
        architectureSchema: InstallerSchema.Definitions.Architecture
    ): Pair<Validation, String?> {
        return when {
            architecture.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Architecture)
            !architectureSchema.enum.contains(architecture) -> {
                Validation.InvalidArchitecture to Errors.invalidEnum(
                    Validation.InvalidArchitecture,
                    architectureSchema.enum
                )
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

    private fun architectureInfo(architectureSchema: InstallerSchema.Definitions.Architecture): String {
        return "${Prompts.required} Enter the architecture. Options: ${architectureSchema.enum.joinToString(", ")}"
    }
}
