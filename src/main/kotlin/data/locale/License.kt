package data.locale

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object License : KoinComponent {
    fun Terminal.licensePrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val licenseSchema = get<SchemasImpl>().defaultLocaleSchema.properties.license
        do {
            println(brightGreen(licenseInfo(licenseSchema)))
            val input = prompt(brightWhite(PromptType.License.toString()))?.trim()
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

    private fun licenseInfo(licenseSchema: DefaultLocaleSchema.Properties.License): String {
        return buildString {
            append(Prompts.required)
            append(" Enter ")
            append(licenseSchema.description.lowercase())
            append(". For example: MIT, GPL, Freeware, Proprietary")
        }
    }
}
