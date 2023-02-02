package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object License : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val licenseSchema = get<SchemasImpl>().defaultLocaleSchema.properties.license
    private val sharedManifestData: SharedManifestData by inject()

    suspend fun Terminal.licensePrompt() {
        sharedManifestData.gitHubDetection?.license?.await()?.let {
            defaultLocaleManifestData.license = it
            return
        }
        println(colors.brightGreen(licenseInfo))
        info(example)
        defaultLocaleManifestData.license = prompt(
            prompt = const,
            default = previousManifestData.remoteDefaultLocaleData?.license?.also { muted("Previous license: $it") },
            convert = { input ->
                isLicenseValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isLicenseValid(license: String): String? {
        return when {
            license.isBlank() -> Errors.blankInput(const)
            license.length < licenseSchema.minLength || license.length > licenseSchema.maxLength -> {
                Errors.invalidLength(min = licenseSchema.minLength, max = licenseSchema.maxLength)
            }
            else -> null
        }
    }

    private val const = DefaultLocaleManifest::license.name.replaceFirstChar { it.titlecase() }
    private val licenseInfo = "${Prompts.required} Enter ${licenseSchema.description.lowercase()}"
    private const val example = "Example: MIT, GPL-3.0, Freeware, Proprietary"
}
