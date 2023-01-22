package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import kotlin.system.exitProcess

object Author : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val authorSchema = get<SchemasImpl>().defaultLocaleSchema.properties.author

    fun Terminal.authorPrompt() {
        println(colors.brightYellow(authorInfo))
        defaultLocaleManifestData.author = prompt(
            prompt = const,
            default = previousManifestData.remoteDefaultLocaleData?.author?.also { muted("Previous author: $it") },
            convert = { input ->
                isAuthorValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isAuthorValid(author: String): String? {
        return when {
            author.isNotBlank() &&
                (author.length < authorSchema.minLength || author.length > authorSchema.maxLength) -> {
                Errors.invalidLength(min = authorSchema.minLength, max = authorSchema.maxLength)
            }
            else -> null
        }
    }

    private const val const = "Author"
    private val authorInfo = "${Prompts.optional} Enter ${authorSchema.description.lowercase()}"
}
