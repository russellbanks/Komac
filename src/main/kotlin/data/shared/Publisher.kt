package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import detection.files.msi.Msi
import detection.files.msix.Msix
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import schemas.manifest.DefaultLocaleManifest

class Publisher(
    private val msi: Msi?,
    private val msix: Msix?,
    private val previousPublisher: String?
) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        return msi?.manufacturer ?: msix?.publisherDisplayName ?: let {
            println(colors.brightGreen(publisherInfo))
            info(example)
            prompt(
                prompt = const,
                default = previousPublisher?.also { muted("Previous publisher: $it") }
            ) { input ->
                getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
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

    class Url(
        private val previousPublisherUrl: io.ktor.http.Url?,
        private val client: HttpClient
    ) : CommandPrompt<io.ktor.http.Url> {
        override fun prompt(terminal: Terminal): io.ktor.http.Url? = with(terminal) {
            println(colors.brightYellow("${Prompts.optional} Enter the publisher home page"))
            return prompt(
                prompt = "Publisher url",
                default = previousPublisherUrl?.also { muted("Previous publisher url: $it") }
            ) { input ->
                runBlocking { data.shared.Url.isUrlValid(url = input.trim().let(::Url), canBeBlank = true, client) }
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().let(::Url))
            }
        }

        override fun getError(input: String?): String? = runBlocking {
            data.shared.Url.isUrlValid(url = input?.trim()?.let(::Url), canBeBlank = true, client)
        }
    }

    private val const = DefaultLocaleManifest::publisher.name.replaceFirstChar(Char::titlecase)

    companion object {
        private const val publisherInfo = "${Prompts.required} Enter the publisher name"
        private const val example = "Example: Microsoft Corporation"
        private const val minLength = 2
        private const val maxLength = 256
    }
}
