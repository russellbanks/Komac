package data.locale

import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import input.ExitCode
import input.Prompts
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import kotlin.system.exitProcess

object ReleaseNotesUrl : KoinComponent, CommandPrompt<Url> {
    override suspend fun prompt(terminal: Terminal): Url = with(terminal) {
        return get<AllManifestData>().gitHubDetection?.releaseNotesUrl ?: let {
            println(colors.brightYellow("${Prompts.optional} Enter the package release notes url"))
            prompt(
                prompt = "Release notes url",
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
