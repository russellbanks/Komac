package data.shared

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
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Publisher : KoinComponent, CommandPrompt<String> {
    private val remoteDefaultLocaleData = get<PreviousManifestData>().remoteDefaultLocaleData
    private val sharedManifestData: SharedManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        return sharedManifestData.msi?.manufacturer ?: sharedManifestData.msix?.publisherDisplayName ?: let {
            println(colors.brightGreen(publisherInfo))
            info(example)
            prompt(
                prompt = const,
                default = remoteDefaultLocaleData.await()?.publisher?.also { muted("Previous publisher: $it") },
                convert = { input ->
                    getError(input)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
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
            return sharedManifestData.gitHubDetection?.publisherUrl ?: let {
                println(colors.brightYellow("${Prompts.optional} Enter the publisher home page"))
                prompt(
                    prompt = "Publisher url",
                    default = remoteDefaultLocaleData.await()?.publisherUrl
                        ?.also { muted("Previous publisher url: $it") },
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

    private val const = DefaultLocaleManifest::publisher.name.replaceFirstChar { it.titlecase() }
    private const val publisherInfo = "${Prompts.required} Enter the publisher name"
    private const val example = "Example: Microsoft Corporation"
    private const val minLength = 2
    private const val maxLength = 256
}
