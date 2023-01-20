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
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.InstallerSchema
import schemas.manifest.InstallerManifest

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
        val detectedArchitectureFromUrl = Ktor.detectArchitectureFromUrl(installerManifestData.installerUrl)
        do {
            architectureInfo().also { (info, infoColor) -> println(infoColor(info)) }
            info("Options: ${architectureSchema.enum.joinToString(", ")}")
            detectedArchitectureFromUrl?.let { println(brightYellow("Detected from Url: $it")) }
            val input = prompt(
                prompt = brightWhite(PromptType.Architecture.toString()),
                default = getPreviousValue()?.also {
                    println(gray("Previous architecture: $it"))
                } ?: detectedArchitectureFromUrl?.toString()
            )!!.trim().lowercase()
            val error = isArchitectureValid(input, architectureSchema)?.also { danger(it) }
            if (error == null) {
                installerManifestData.architecture = input.toArchitecture()
            }
            println()
        } while (error != null)
    }

    private fun isArchitectureValid(
        architecture: String?,
        architectureSchema: InstallerSchema.Definitions.Architecture
    ): String? {
        return when {
            architecture.isNullOrBlank() -> Errors.blankInput(PromptType.Architecture)
            !architectureSchema.enum.contains(architecture) -> {
                Errors.invalidEnum(
                    Validation.InvalidArchitecture,
                    architectureSchema.enum
                )
            }
            else -> null
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
