package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import io.ktor.http.Url
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object Architecture : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val architectureSchema = schemasImpl.installerSchema.definitions.architecture
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.architecturePrompt() {
        sharedManifestData.msix?.processorArchitecture?.let {
            installerManifestData.architecture = it
            return
        }
        sharedManifestData.msixBundle?.packages?.first()?.processorArchitecture?.let {
            installerManifestData.architecture = it
            return
        }
        val detectedArchitectureFromUrl = Ktor.detectArchitectureFromUrl(Url(installerManifestData.installerUrl))
        do {
            architectureInfo().also { (info, infoColor) -> println(infoColor(info)) }
            println(cyan("Options: ${architectureSchema.enum.joinToString(", ")}"))
            detectedArchitectureFromUrl?.let { println(brightYellow("Detected from Url: $it")) }
            val input = prompt(
                prompt = brightWhite(PromptType.Architecture.toString()),
                default = getPreviousValue()?.also {
                    println(gray("Previous architecture: $it"))
                } ?: detectedArchitectureFromUrl?.toString()
            )?.trim()?.lowercase()
            val (architectureValid, error) = isArchitectureValid(input, architectureSchema)
            error?.let { println(brightRed(it)) }
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

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.installers
            ?.get(installerManifestData.installers.size)?.architecture?.toString()
    }

    private fun architectureInfo(): Pair<String, TextColors> {
        return buildString {
            append(if (getPreviousValue() == null) Prompts.required else Prompts.optional)
            append(" Enter the architecture")
        } to if (getPreviousValue() == null) brightGreen else brightYellow
    }
}
