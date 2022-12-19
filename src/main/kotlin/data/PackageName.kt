package data

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object PackageName : KoinComponent {
    fun Terminal.packageNamePrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val packageNameSchema = schemasImpl.defaultLocaleSchema.properties.packageName
        do {
            println(brightGreen(packageNameInfo(packageNameSchema)))
            val input = prompt(brightWhite(PromptType.PackageName.toString()))?.trim()
            val (packageNameValid, error) = packageNameValid(input, packageNameSchema)
            if (packageNameValid == Validation.Success && input != null) {
                defaultLocaleManifestData.packageName = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageNameValid != Validation.Success)
    }

    fun packageNameValid(
        input: String?,
        packageNameSchema: DefaultLocaleSchema.Properties.PackageName
    ): Pair<Validation, String?> {
        return when {
            input.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Publisher)
            input.length < packageNameSchema.minLength || input.length > packageNameSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = packageNameSchema.minLength,
                    max = packageNameSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun packageNameInfo(packageNameSchema: DefaultLocaleSchema.Properties.PackageName): String {
        return "Enter ${packageNameSchema.description.lowercase()}. For example, Microsoft Teams"
    }
}
