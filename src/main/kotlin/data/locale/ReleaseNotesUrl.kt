package data.locale

import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking

class ReleaseNotesUrl(private val client: HttpClient) : CommandPrompt<Url> {
    override fun prompt(terminal: Terminal): Url? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Enter the package release notes url"))
        return prompt("Release notes url") { input ->
            runBlocking { data.shared.Url.isUrlValid(url = Url(input.trim()), canBeBlank = true, client) }
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(Url(input.trim()))
        }
    }

    override fun getError(input: String?): String? = runBlocking {
        data.shared.Url.isUrlValid(url = input?.trim()?.let(::Url), canBeBlank = true, client)
    }
}
