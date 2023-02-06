package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.DefaultLocaleManifestData
import input.ExitCode
import input.Prompts
import kotlin.system.exitProcess

object Copyright : CommandPrompt<String> {
    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        println(colors.brightYellow(copyrightInfo))
        info(example)
        return prompt(
            prompt = DefaultLocaleManifestData::copyright.name.replaceFirstChar { it.titlecase() },
            convert = { input ->
                getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ).also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isNotBlank() &&
                (input.length < minLength || input.length > maxLength) -> {
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
