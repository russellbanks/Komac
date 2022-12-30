package data.shared

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.InstallerSchema
import schemas.SchemasImpl

object Locale : KoinComponent {
    val installerManifestData: InstallerManifestData by inject()
    val sharedManifestData: SharedManifestData by inject()
    val previousManifestData: PreviousManifestData by inject()

    fun Terminal.localePrompt(promptType: PromptType) {
        do {
            localeInfo(promptType).also { (info, infoColor) -> println(infoColor(info)) }
            if (promptType == PromptType.InstallerLocale) println(cyan("Example: en-US"))
            val input = prompt(
                prompt = brightWhite(promptType.toString()),
                default = when (promptType) {
                    PromptType.InstallerType -> getPreviousValue()?.also { println(gray("Previous value: $it")) }
                    PromptType.PackageLocale -> get<SchemasImpl>().defaultLocaleSchema.properties.packageLocale.default
                    else -> null
                }
            )?.trim()?.lowercase()
            val (localeValid, error) = if (promptType == PromptType.InstallerLocale) {
                isInstallerLocaleValid(input)
            } else {
                isPackageLocaleValid(input)
            }
            error?.let { println(red(it)) }
            if (localeValid == Validation.Success && input != null) {
                if (promptType == PromptType.InstallerLocale) {
                    installerManifestData.installerLocale = input
                } else {
                    sharedManifestData.defaultLocale = input
                }
            }
            println()
        } while (localeValid != Validation.Success)
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

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerLocale ?: it.installers[installerManifestData.installers.size].installerLocale
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

    private fun localeInfo(promptType: PromptType): Pair<String, TextColors> {
        return buildString {
            append(if (promptType == PromptType.PackageLocale) Prompts.required else Prompts.optional)
            append(" Enter the $promptType")
        } to if (promptType == PromptType.PackageLocale) brightGreen else brightYellow
    }
}
