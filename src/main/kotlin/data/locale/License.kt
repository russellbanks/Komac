package data.locale

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl

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
        do {
            println(brightGreen(licenseInfo))
            println(cyan(licenseExample))
            val input = prompt(
                prompt = brightWhite(PromptType.License.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.license?.also {
                    println(gray("Previous license: $it"))
                }
            )?.trim()
            val error = isLicenseValid(input)
            if (error == null && input != null) {
                defaultLocaleManifestData.license = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (error != null)
    }

    private fun isLicenseValid(license: String?): String? {
        return when {
            license.isNullOrBlank() -> Errors.blankInput(PromptType.License)
            license.length < licenseSchema.minLength || license.length > licenseSchema.maxLength -> {
                Errors.invalidLength(min = licenseSchema.minLength, max = licenseSchema.maxLength)
            }
            else -> null
        }
    }

    private val licenseInfo = "${Prompts.required} Enter ${licenseSchema.description.lowercase()}"
    private const val licenseExample = "Example: MIT, GPL, Freeware, Proprietary"
}
