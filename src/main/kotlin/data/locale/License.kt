package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object License : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val licenseSchema = get<SchemasImpl>().defaultLocaleSchema.properties.license

    fun Terminal.licensePrompt() {
        do {
            println(brightGreen(licenseInfo))
            println(cyan(licenseExample))
            val input = prompt(
                prompt = brightWhite(PromptType.License.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.license?.also {
                    println(gray("Previous license: $it"))
                }
            )?.trim()
            val (packageLocaleValid, error) = isLicenseValid(input, licenseSchema)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.license = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageLocaleValid != Validation.Success)
    }

    fun isLicenseValid(
        license: String?,
        licenseSchema: DefaultLocaleSchema.Properties.License
    ): Pair<Validation, String?> {
        return when {
            license.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.License)
            license.length < licenseSchema.minLength || license.length > licenseSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = licenseSchema.minLength,
                    max = licenseSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private val licenseInfo = "${Prompts.required} Enter ${licenseSchema.description.lowercase()}"
    private const val licenseExample = "Example: MIT, GPL, Freeware, Proprietary"
}
