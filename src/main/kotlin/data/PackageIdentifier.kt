package data

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
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl
import schemas.Pattern

object PackageIdentifier : KoinComponent {
    suspend fun Terminal.packageIdentifierPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl = get()
        do {
            println(brightGreen(Prompts.packageIdentifierInfo))
            installerManifestData.packageIdentifier = prompt(brightWhite(Prompts.packageIdentifier))?.trim()
            installerSchemaImpl.awaitInstallerSchema()
            val (packageIdentifierValid, error) = isPackageIdentifierValid(
                installerManifestData.packageIdentifier
            )
            error?.let { println(red(it)) }
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    fun isPackageIdentifierValid(
        identifier: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageIdentifierMaxLength = installerSchema.definitions.packageIdentifier.maxLength
        val packageIdentifierRegex = Pattern.packageIdentifier(installerSchema)
        return when {
            identifier.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            identifier.length > packageIdentifierMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = packageIdentifierMinLength,
                    max = packageIdentifierMaxLength
                )
            }
            !identifier.matches(packageIdentifierRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(packageIdentifierRegex)
            }
            else -> Validation.Success to null
        }
    }

    private const val packageIdentifierMinLength = 4
}
