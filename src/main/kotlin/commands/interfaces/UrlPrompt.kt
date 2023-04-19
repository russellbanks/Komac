package commands.interfaces

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking

interface UrlPrompt : Prompt<Url> {
    val name: String

    val description: String

    val previousUrl: Url? get() = null

    val transform: (String) -> Url get() = { urlString -> Url(urlString.trim()) }

    override suspend fun prompt(terminal: Terminal): Url = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Enter the $description"))
        return prompt(
            prompt = name,
            default = previousUrl?.also { muted("Previous ${name.lowercase()}: $it") }
        ) { input ->
            runBlocking { getError(input) }
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(transform(input))
        } ?: throw ProgramResult(0)
    }

    override suspend fun getError(input: String): String? {
        return data.shared.Url.isUrlValid(transform(input), canBeBlank = true)
    }
}
