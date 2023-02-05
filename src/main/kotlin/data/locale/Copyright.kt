package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import kotlin.system.exitProcess

object Copyright : KoinComponent {
    fun Terminal.copyrightPrompt() {
        println(colors.brightYellow(copyrightInfo))
        info(example)
        get<DefaultLocaleManifestData>().copyright = prompt(
            prompt = DefaultLocaleManifestData::copyright.name.replaceFirstChar { it.titlecase() },
            convert = { input ->
                isCopyrightValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isCopyrightValid(copyright: String): String? {
        return when {
            copyright.isNotBlank() &&
                (copyright.length < minLength || copyright.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    private const val copyrightInfo = "${Prompts.optional} Enter the package copyright"
    private const val example = "Example: Copyright (c) Microsoft Corporation"
    private const val minLength = 3
    private const val maxLength = 512
}
