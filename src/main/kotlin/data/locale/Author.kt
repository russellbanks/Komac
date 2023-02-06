package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import kotlin.system.exitProcess

object Author : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.authorPrompt() {
        println(colors.brightYellow(authorInfo))
        defaultLocaleManifestData.author = prompt(
            prompt = DefaultLocaleManifestData::author.name.replaceFirstChar { it.titlecase() },
            default = previousManifestData.remoteDefaultLocaleData?.author?.also { muted("Previous author: $it") },
            convert = { input ->
                isAuthorValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isAuthorValid(author: String): String? {
        return when {
            author.isNotBlank() && (author.length < minLength || author.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    private const val authorInfo = "${Prompts.optional} Enter the package author"
    private const val minLength = 2
    private const val maxLength = 256
}
