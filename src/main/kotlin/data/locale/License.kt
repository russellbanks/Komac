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

class License(private val previousLicense: String?) : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightGreen(licenseInfo))
        info(example)
        return prompt(
            prompt = const,
            default = previousLicense?.also { muted("Previous license: $it") }
        ) { input ->
            getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
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
        private val previousLicenseUrl: io.ktor.http.Url?,
        private val client: HttpClient
    ) : CommandPrompt<io.ktor.http.Url> {
        override fun prompt(terminal: Terminal): io.ktor.http.Url? = with(terminal) {
            println(colors.brightYellow("${Prompts.optional} Enter the license page url"))
            return prompt(
                prompt = "License url",
                default = previousLicenseUrl?.also { muted("Previous license url: $it") }
            ) { input ->
                runBlocking { data.shared.Url.isUrlValid(url = Url(input.trim()), canBeBlank = true, client) }
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(Url(input.trim()))
            }
        }

        override fun getError(input: String?): String? = runBlocking {
            data.shared.Url.isUrlValid(url = input?.trim()?.let(::Url), canBeBlank = true, client)
        }
    }

    private val const = DefaultLocaleManifest::license.name.replaceFirstChar(Char::titlecase)

    companion object {
        private const val licenseInfo = "${Prompts.required} Enter the package license"
        private const val example = "Example: MIT, GPL-3.0, Freeware, Proprietary"
        private const val minLength = 3
        private const val maxLength = 512
    }
}
