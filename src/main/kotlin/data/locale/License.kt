package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
import input.Prompts
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object License : KoinComponent, CommandPrompt<String> {
    private val gitHubDetection = get<SharedManifestData>().gitHubDetection
    private val remoteDefaultLocaleData = get<PreviousManifestData>().remoteDefaultLocaleData

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        return gitHubDetection?.license?.await() ?: let {
            println(colors.brightGreen(licenseInfo))
            info(example)
            prompt(
                prompt = const,
                default = remoteDefaultLocaleData.await()?.license?.also { muted("Previous license: $it") },
                convert = { input ->
                    getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
                }
            )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length < minLength || input.length > maxLength -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    object Url : CommandPrompt<io.ktor.http.Url> {
        override suspend fun prompt(terminal: Terminal): io.ktor.http.Url = with(terminal) {
            return gitHubDetection?.licenseUrl?.await() ?: let {
                println(colors.brightYellow("${Prompts.optional} Enter the license page url"))
                prompt(
                    prompt = "License url",
                    default = remoteDefaultLocaleData.await()?.licenseUrl?.also { muted("Previous license url: $it") },
                    convert = { input ->
                        runBlocking { data.shared.Url.isUrlValid(url = Url(input.trim()), canBeBlank = true) }
                            ?.let { ConversionResult.Invalid(it) }
                            ?: ConversionResult.Valid(Url(input.trim()))
                    }
                )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
            }
        }

        override fun getError(input: String?): String? = runBlocking {
            data.shared.Url.isUrlValid(url = input?.trim()?.let { Url(it) }, canBeBlank = true)
        }
    }

    private val const = DefaultLocaleManifest::license.name.replaceFirstChar { it.titlecase() }
    private const val licenseInfo = "${Prompts.required} Enter the package license"
    private const val example = "Example: MIT, GPL-3.0, Freeware, Proprietary"
    private const val minLength = 3
    private const val maxLength = 512
}
