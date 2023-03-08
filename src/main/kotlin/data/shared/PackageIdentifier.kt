package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts

object PackageIdentifier : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightGreen(identifierInfo))
        info(example)
        return prompt(const) { input ->
            getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length > maxLength -> Errors.invalidLength(min = minLength, max = maxLength)
            !input.matches(regex) -> Errors.invalidRegex(regex)
            else -> null
        }
    }

    private const val const = "Package Identifier"
    private const val example = "Example: Microsoft.Excel"
    private const val identifierInfo = "${Prompts.required} Enter the $const, " +
        "in the following format <Publisher shortname.Application shortname>"
    const val maxLength = 128
    const val minLength = 4
    private const val pattern = "^[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}(\\.[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}){1,7}$"
    private val regex = Regex(pattern)
}
