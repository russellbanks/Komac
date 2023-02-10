package data.locale

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
import kotlin.system.exitProcess

object PackageUrl : KoinComponent, CommandPrompt<Url> {
    val remoteDefaultLocaleData = get<PreviousManifestData>().remoteDefaultLocaleData

    override suspend fun prompt(terminal: Terminal): Url = with(terminal) {
        return get<SharedManifestData>().gitHubDetection?.packageUrl?.await() ?: let {
            println(colors.brightYellow("${Prompts.optional} Enter the package home page"))
            return prompt(
                prompt = "Package Url",
                default = remoteDefaultLocaleData.await()?.packageUrl?.also { muted("Previous package url: $it") },
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
