package data

import Errors
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
import schemas.Schema
import schemas.SchemasImpl

object PackageIdentifier : KoinComponent {
    suspend fun Terminal.packageIdentifierPrompt() {
        val sharedManifestData: SharedManifestData by inject()
        val schemasImpl: SchemasImpl = get()
        do {
            println(brightGreen(packageIdentifierInfo))
            val input = prompt(brightWhite(PromptType.PackageIdentifier.toString()))?.trim()
            schemasImpl.awaitSchema(Schema.Installer)
            val (packageIdentifierValid, error) = isPackageIdentifierValid(input)
            if (packageIdentifierValid == Validation.Success && input != null) {
                sharedManifestData.packageIdentifier = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    fun isPackageIdentifierValid(
        identifier: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageIdentifierSchema = installerSchema.definitions.packageIdentifier
        return when {
            identifier.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageIdentifier)
            identifier.length > packageIdentifierSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = packageIdentifierMinLength,
                    max = packageIdentifierSchema.maxLength
                )
            }
            !identifier.matches(Regex(packageIdentifierSchema.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(packageIdentifierSchema.pattern))
            }
            else -> Validation.Success to null
        }
    }

    private const val packageIdentifierInfo = "${Prompts.required} Enter the Package Identifier, " +
        "in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"
    private const val packageIdentifierMinLength = 4
}
