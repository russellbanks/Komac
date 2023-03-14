package data.shared

import Errors
import com.github.ajalt.mordant.rendering.TextStyle
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import input.LocaleType
import input.Prompts
import schemas.manifest.InstallerManifest
import java.util.Locale

object Locale {
    const val installerLocaleConst = "Installer Locale"
    private const val maxLength = 20
    private const val pattern = "^([a-zA-Z]{2,3}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*$"
    private val regex = Regex(pattern)
    const val defaultLocale = "en-US"

    class Installer(
        private val allManifestData: AllManifestData,
        private val previousInstallerManifest: InstallerManifest?
    ) : CommandPrompt<String> {
        override fun prompt(terminal: Terminal): String? = with(terminal) {
            return allManifestData.msi?.productLanguage ?: let {
                localeInfo(LocaleType.Installer).also { (info, infoColor) -> println(infoColor(info)) }
                info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
                prompt(
                    prompt = installerLocaleConst,
                    default = getPreviousValue()?.also { muted("Previous value: $it") }
                ) { input ->
                    getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
                }
            }
        }

        override fun getError(input: String?): String? = getError(input, LocaleType.Installer)

        private fun getPreviousValue(): String? {
            return previousInstallerManifest?.let {
                it.installerLocale ?: it.installers[allManifestData.installers.size].installerLocale
            }
        }
    }

    class Package(private val previousPackageLocale: String?) : CommandPrompt<String> {
        override fun prompt(terminal: Terminal): String? = with(terminal) {
            localeInfo(LocaleType.Package).also { (info, infoColor) -> println(infoColor(info)) }
            info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
            prompt(
                prompt = "Package locale",
                default = previousPackageLocale ?: defaultLocale,
                convert = { input ->
                    getError(input.trim())
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            )
        }

        override fun getError(input: String?) = getError(input, LocaleType.Package)
    }

    fun getError(input: String?, localeType: LocaleType): String? {
        return when {
            input == null -> null
            input.isBlank() -> if (localeType == LocaleType.Package) Errors.blankInput(localeType.toString()) else null
            !input.matches(regex) -> Errors.invalidRegex(regex)
            input.length > maxLength -> Errors.invalidLength(max = maxLength)
            else -> null
        }
    }

    private fun Terminal.localeInfo(localeType: LocaleType): Pair<String, TextStyle> {
        return buildString {
            append(if (localeType == LocaleType.Package) Prompts.required else Prompts.optional)
            append(" Enter the $localeType locale")
        } to if (localeType == LocaleType.Package) colors.brightGreen else colors.brightYellow
    }
}
