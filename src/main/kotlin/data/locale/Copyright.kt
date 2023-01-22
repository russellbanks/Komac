package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import kotlin.system.exitProcess

object Copyright : KoinComponent {
    private val copyrightSchema = get<SchemasImpl>().defaultLocaleSchema.properties.copyright

    fun Terminal.copyrightPrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        println(colors.brightYellow(copyrightInfo))
        info(example)
        defaultLocaleManifestData.copyright = prompt(
            prompt = const,
            convert = { input ->
                isCopyrightValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isCopyrightValid(copyright: String): String? {
        return when {
            copyright.isNotBlank() &&
                (copyright.length < copyrightSchema.minLength || copyright.length > copyrightSchema.maxLength) -> {
                Errors.invalidLength(min = copyrightSchema.minLength, max = copyrightSchema.maxLength)
            }
            else -> null
        }
    }

    private const val const = "Copyright"
    private val copyrightInfo = "${Prompts.optional} Enter ${copyrightSchema.description.lowercase()}"
    private const val example = "Example: Copyright (c) Microsoft Corporation"
}
