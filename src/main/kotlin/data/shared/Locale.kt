package data.shared

import Errors
import com.github.ajalt.mordant.rendering.TextStyle
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
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

    object Installer : CommandPrompt<String> {
        override suspend fun prompt(terminal: Terminal): String = with(terminal) {
            return sharedManifestData.msi?.productLanguage ?: let {
                localeInfo(LocaleType.Installer).also { (info, infoColor) -> println(infoColor(info)) }
                info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
                prompt(
                    prompt = "Installer locale",
                    default = getPreviousValue()?.also { muted("Previous value: $it") },
                    convert = { input ->
                        getError(input.trim())
                            ?.let { ConversionResult.Invalid(it) }
                            ?: ConversionResult.Valid(input.trim())
                    }
                )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
            }
        }

        override fun getError(input: String?): String? = getError(input, LocaleType.Installer)
    }

    object Package : CommandPrompt<String> {
        override suspend fun prompt(terminal: Terminal): String = with(terminal) {
            localeInfo(LocaleType.Package).also { (info, infoColor) -> println(infoColor(info)) }
            info("Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}")
            prompt(
                prompt = "Package locale",
                default = defaultLocale,
                convert = { input ->
                    getError(input.trim())
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
        }

        override fun getError(input: String?) = getError(input, LocaleType.Package)
    }

    fun getError(input: String?, localeType: LocaleType): String? {
        return when {
            input == null -> null
            input.isBlank() -> if (localeType == LocaleType.Package) Errors.blankInput(localeType) else null
            !input.matches(regex) -> Errors.invalidRegex(regex)
            input.length > maxLength -> Errors.invalidLength(max = maxLength)
            else -> null
        }
    }

    private suspend fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.installerLocale ?: it.installers[installerManifestData.installers.size].installerLocale
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
