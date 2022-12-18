package data

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
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

object InstallerLocale : KoinComponent {
    fun Terminal.installerLocalePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightYellow(Prompts.installerLocaleInfo))
            installerManifestData.installerLocale = prompt(brightWhite(PromptType.InstallerLocale.toString()))?.trim()
            val (installerLocaleValid, error) = isInstallerLocaleValid(installerManifestData.installerLocale)
            error?.let { println(red(it)) }
            println()
        } while (installerLocaleValid != Validation.Success)
    }

    fun isInstallerLocaleValid(
        locale: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerLocaleMaxLength = installerSchema.definitions.locale.maxLength
        val installerLocaleRegex = Pattern.installerLocale(installerSchema)
        return when {
            !locale.isNullOrBlank() && !locale.matches(installerLocaleRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(installerLocaleRegex)
            }
            (locale?.length ?: 0) > installerLocaleMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerLocaleMaxLength)
            }
            else -> Validation.Success to null
        }
    }
}
