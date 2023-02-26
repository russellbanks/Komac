package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Author : KoinComponent, CommandPrompt<String> {
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        println(colors.brightYellow(authorInfo))
        return prompt(
            prompt = DefaultLocaleManifest::author.name.replaceFirstChar { it.titlecase() },
            default = previousManifestData.remoteDefaultLocaleData.await()?.author
                ?.also { muted("Previous author: $it") },
            convert = { input ->
                getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ).also { println() } ?: exitProcess(ExitCode.CtrlC.code)
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

    private const val authorInfo = "${Prompts.optional} Enter the package author"
    private const val minLength = 2
    private const val maxLength = 256
}
