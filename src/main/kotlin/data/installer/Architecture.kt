package data.installer

import Errors
import ExitCode
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

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
        architectureInfo().also { (info, infoColor) -> println(infoColor(info)) }
        info("Options: ${architectureSchema.enum.joinToString(", ")}")
        detectedArchitectureFromUrl?.let { info("Detected from Url: $it") }
        installerManifestData.architecture = prompt(
            prompt = const,
            default = getPreviousValue()?.toArchitecture()?.also { muted("Previous architecture: $it") }
                ?: detectedArchitectureFromUrl,
            convert = { input ->
                isArchitectureValid(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().toArchitecture())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isArchitectureValid(architecture: String): String? {
        return when {
            architecture.isBlank() -> Errors.blankInput(const)
            !architectureSchema.enum.contains(architecture) -> {
                Errors.invalidEnum(Validation.InvalidArchitecture, architectureSchema.enum)
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

    const val const = "Architecture"
}
