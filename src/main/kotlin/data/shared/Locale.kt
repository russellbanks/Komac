package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import java.util.Locale
import kotlin.system.exitProcess

object Locale : KoinComponent {
    val installerManifestData: InstallerManifestData by inject()
    val sharedManifestData: SharedManifestData by inject()
    val previousManifestData: PreviousManifestData by inject()

    fun Terminal.localePrompt(promptType: PromptType) {
        if (promptType == PromptType.InstallerLocale) {
            sharedManifestData.msi?.productLanguage?.let {
                installerManifestData.installerLocale = it
                return
            }
        }
        localeInfo(promptType).also { (info, infoColor) -> println(infoColor(info)) }
        info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
        val input = prompt(
            prompt = brightWhite(promptType.toString()),
            default = when (promptType) {
                PromptType.InstallerLocale -> getPreviousValue()?.also { muted("Previous value: $it") }
                PromptType.PackageLocale -> get<SchemasImpl>().defaultLocaleSchema.properties.packageLocale.default
                else -> null
            },
            convert = { input ->
                val error = if (promptType == PromptType.InstallerLocale) {
                    isInstallerLocaleValid(input)
                } else {
                    isPackageLocaleValid(input)
                }
                error?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        if (promptType == PromptType.InstallerLocale) {
            installerManifestData.installerLocale = input
        } else {
            sharedManifestData.defaultLocale = input
        }
        println()
    }

    private fun isInstallerLocaleValid(locale: String): String? {
        val installerLocale = get<SchemasImpl>().installerSchema.definitions.locale
        return when {
            locale.isNotBlank() && !locale.matches(Regex(installerLocale.pattern)) -> {
                Errors.invalidRegex(Regex(installerLocale.pattern))
            }
            locale.length > installerLocale.maxLength -> Errors.invalidLength(max = installerLocale.maxLength)
            else -> null
        }
    }

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerLocale ?: it.installers[installerManifestData.installers.size].installerLocale
        }
    }

    private fun isPackageLocaleValid(locale: String): String? {
        val packageLocaleSchema = get<SchemasImpl>().defaultLocaleSchema.properties.packageLocale
        return when {
            locale.isBlank() -> Errors.blankInput(PromptType.PackageLocale)
            !locale.matches(Regex(packageLocaleSchema.pattern)) -> {
                Errors.invalidRegex(Regex(packageLocaleSchema.pattern))
            }
            locale.length > packageLocaleSchema.maxLength -> Errors.invalidLength(max = packageLocaleSchema.maxLength)
            else -> null
        }
    }

    private fun localeInfo(promptType: PromptType): Pair<String, TextColors> {
        return buildString {
            append(if (promptType == PromptType.PackageLocale) Prompts.required else Prompts.optional)
            append(" Enter the $promptType")
        } to if (promptType == PromptType.PackageLocale) brightGreen else brightYellow
    }

    const val installerLocaleConst = "Installer Locale"
}
