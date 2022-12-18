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

object PackageVersion : KoinComponent {
    fun Terminal.packageVersionPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            installerManifestData.packageVersion = prompt(brightWhite(Prompts.packageVersion))?.trim()
            val (packageVersionValid, error) = isPackageVersionValid(installerManifestData.packageVersion)
            error?.let { println(red(it)) }
            println()
        } while (packageVersionValid != Validation.Success)
    }

    fun isPackageVersionValid(
        version: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageVersionMaxLength = installerSchema.definitions.packageVersion.maxLength
        val packageVersionRegex = Pattern.packageVersion(installerSchema)
        return when {
            version.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            version.length > packageVersionMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = packageVersionMaxLength)
            }
            !version.matches(packageVersionRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(packageVersionRegex)
            }
            else -> Validation.Success to null
        }
    }
}
