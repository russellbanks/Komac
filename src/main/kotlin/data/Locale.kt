package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.InstallerSchema
import schemas.SchemasImpl

object Locale : KoinComponent {
    fun Terminal.installerLocalePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightYellow(localeInfo(PromptType.InstallerLocale)))
            installerManifestData.installerLocale = prompt(brightWhite(PromptType.InstallerLocale.toString()))?.trim()
            val (installerLocaleValid, error) = isInstallerLocaleValid(installerManifestData.installerLocale)
            error?.let { println(red(it)) }
            println()
        } while (installerLocaleValid != Validation.Success)
    }

    suspend fun Terminal.packageLocalePrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val schemasImpl: SchemasImpl = get()
        schemasImpl.awaitDefaultLocaleSchema()
        val packageLocaleSchema = schemasImpl.defaultLocaleSchema.properties.packageLocale
        do {
            println(brightGreen(localeInfo(PromptType.PackageLocale)))
            val input = prompt(
                prompt = brightWhite(PromptType.PackageLocale.toString()),
                default = packageLocaleSchema.default
            )?.trim()
            val (packageLocaleValid, error) = isPackageLocaleValid(input)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.packageLocale = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageLocaleValid != Validation.Success)
    }

    fun isInstallerLocaleValid(
        locale: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerLocale = installerSchema.definitions.locale
        return when {
            !locale.isNullOrBlank() && !locale.matches(Regex(installerLocale.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(installerLocale.pattern))
            }
            (locale?.length ?: 0) > installerLocale.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerLocale.maxLength)
            }
            else -> Validation.Success to null
        }
    }

    fun isPackageLocaleValid(
        locale: String?,
        defaultLocaleSchema: DefaultLocaleSchema = get<SchemasImpl>().defaultLocaleSchema
    ): Pair<Validation, String?> {
        val packageLocaleSchema = defaultLocaleSchema.properties.packageLocale
        return when {
            locale.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageLocale)
            !locale.matches(Regex(packageLocaleSchema.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(packageLocaleSchema.pattern))
            }
            locale.length > packageLocaleSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = packageLocaleSchema.maxLength)
            }
            else -> Validation.Success to null
        }
    }

    private fun localeInfo(promptType: PromptType): String {
        return buildString {
            append(if (promptType == PromptType.PackageLocale) Prompts.required else Prompts.optional)
            append(" Enter the $promptType. For example: en-US, en-CA")
        }
    }
}
