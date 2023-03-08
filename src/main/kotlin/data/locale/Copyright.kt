package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import schemas.manifest.DefaultLocaleManifest

class Copyright(private val previousCopyright: String?) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightYellow(copyrightInfo))
        info(example)
        return prompt(
            prompt = DefaultLocaleManifest::copyright.name.replaceFirstChar(Char::titlecase),
            default = previousCopyright?.also { muted("Previous copyright: $it") }
        ) { input ->
            getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isNotBlank() && (input.length < minLength || input.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }

            else -> null
        }
    }

    class Url(
        private val previousCopyrightUrl: io.ktor.http.Url?,
        private val client: HttpClient
    ) : CommandPrompt<io.ktor.http.Url> {
        override fun prompt(terminal: Terminal): io.ktor.http.Url? = with(terminal) {
            println(colors.brightYellow("${Prompts.optional} Enter the package's copyright url"))
            return prompt(
                prompt = "Copyright url",
                default = previousCopyrightUrl?.also { muted("Previous copyright url: $it") },
                convert = { input ->
                    runBlocking { data.shared.Url.isUrlValid(url = input.trim().let(::Url), canBeBlank = true, client) }
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim().let(::Url))
                }
            )
        }

        override fun getError(input: String?): String? = runBlocking {
            data.shared.Url.isUrlValid(input?.trim()?.let(::Url), canBeBlank = true, client)
        }
    }

    companion object {
        private const val copyrightInfo = "${Prompts.optional} Enter the package copyright"
        private const val example = "Example: Copyright (c) Microsoft Corporation"
        private const val minLength = 3
        private const val maxLength = 512
    }
}
