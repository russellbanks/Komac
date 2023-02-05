package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.rendering.TextStyle
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.LocaleType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.util.Locale
import kotlin.system.exitProcess

object Locale : KoinComponent {
    val installerManifestData: InstallerManifestData by inject()
    val sharedManifestData: SharedManifestData by inject()
    val previousManifestData: PreviousManifestData by inject()

    fun Terminal.localePrompt(localeType: LocaleType) {
        if (localeType == LocaleType.Installer) {
            sharedManifestData.msi?.productLanguage?.let {
                installerManifestData.installerLocale = it
                return
            }
        }
        localeInfo(localeType).also { (info, infoColor) -> println(infoColor(info)) }
        info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
        val input = prompt(
            prompt = "$localeType locale",
            default = when (localeType) {
                LocaleType.Installer -> getPreviousValue()?.also { muted("Previous value: $it") }
                LocaleType.Package -> defaultLocale
            },
            convert = { input ->
                if (localeType == LocaleType.Installer) {
                    isInstallerLocaleValid(input)
                } else {
                    isPackageLocaleValid(input)
                }?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        if (localeType == LocaleType.Installer) {
            installerManifestData.installerLocale = input
        } else {
            sharedManifestData.defaultLocale = input
        }
        println()
    }

    private fun isInstallerLocaleValid(locale: String): String? {
        return when {
            locale.isNotBlank() && !locale.matches(regex) -> Errors.invalidRegex(regex)
            locale.length > maxLength -> Errors.invalidLength(max = maxLength)
            else -> null
        }
    }

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerLocale ?: it.installers[installerManifestData.installers.size].installerLocale
        }
    }

    private fun isPackageLocaleValid(locale: String): String? {
        return when {
            locale.isBlank() -> Errors.blankInput(LocaleType.Package)
            !locale.matches(regex) -> Errors.invalidRegex(regex)
            locale.length > maxLength -> Errors.invalidLength(max = maxLength)
            else -> null
        }
    }

    private fun Terminal.localeInfo(localeType: LocaleType): Pair<String, TextStyle> {
        return buildString {
            append(if (localeType == LocaleType.Package) Prompts.required else Prompts.optional)
            append(" Enter the $localeType locale")
        } to if (localeType == LocaleType.Package) colors.brightGreen else colors.brightYellow
    }

    const val installerLocaleConst = "Installer Locale"
    private const val maxLength = 20
    private const val pattern = "^([a-zA-Z]{2,3}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*$"
    private val regex = Regex(pattern)
    private const val defaultLocale = "en-US"
}
