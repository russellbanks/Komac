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
import schemas.SchemasImpl

object PackageVersion : KoinComponent {
    fun Terminal.packageVersionPrompt() {
        val sharedManifestData: SharedManifestData by inject()
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            val input = prompt(brightWhite(PromptType.PackageVersion.toString()))?.trim()
            val (packageVersionValid, error) = isPackageVersionValid(input)
            if (packageVersionValid == Validation.Success && input != null) {
                sharedManifestData.packageVersion = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageVersionValid != Validation.Success)
    }

    fun isPackageVersionValid(
        version: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageVersionSchema = installerSchema.definitions.packageVersion
        return when {
            version.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            version.length > packageVersionSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = packageVersionSchema.maxLength)
            }
            !version.matches(Regex(packageVersionSchema.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(packageVersionSchema.pattern))
            }
            else -> Validation.Success to null
        }
    }
}
