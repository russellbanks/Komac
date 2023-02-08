package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import kotlin.system.exitProcess

object Copyright : CommandPrompt<String> {
    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        println(colors.brightYellow(copyrightInfo))
        info(example)
        return prompt(
            prompt = DefaultLocaleManifestData::copyright.name.replaceFirstChar { it.titlecase() },
            convert = { input ->
                getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ).also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isNotBlank() &&
                (input.length < minLength || input.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    object Url : CommandPrompt<io.ktor.http.Url>, KoinComponent {
        val remoteDefaultLocaleData = get<PreviousManifestData>().remoteDefaultLocaleData
        override suspend fun prompt(terminal: Terminal): io.ktor.http.Url = with(terminal) {
            println(colors.brightYellow("${Prompts.optional} Enter the package's copyright url"))
            return prompt(
                prompt = "Copyright url",
                default = remoteDefaultLocaleData.await()?.copyrightUrl?.also { muted("Previous copyright url: $it") },
                convert = { input ->
                    runBlocking { data.shared.Url.isUrlValid(url = Url(input.trim()), canBeBlank = true) }
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(Url(input.trim()))
                }
            )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
        }

        override fun getError(input: String?): String? = runBlocking {
            data.shared.Url.isUrlValid(url = input?.trim()?.let { Url(it) }, canBeBlank = true)
        }
    }

    private const val copyrightInfo = "${Prompts.optional} Enter the package copyright"
    private const val example = "Example: Copyright (c) Microsoft Corporation"
    private const val minLength = 3
    private const val maxLength = 512
}
