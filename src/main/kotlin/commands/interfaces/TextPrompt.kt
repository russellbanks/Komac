package commands.interfaces

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import input.ExitCode
import input.Prompts
import kotlinx.coroutines.runBlocking

interface TextPrompt : Prompt<String> {
    val name: String

    val extraText: String? get() = null

    val validationRules: ValidationRules

    val default: String? get() = null

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        val textColour = if (validationRules.isRequired) colors.brightGreen else colors.brightYellow
        val requiredText = if (validationRules.isRequired) Prompts.required else Prompts.optional
        println(textColour("$requiredText Enter the $name"))
        extraText?.let { info(extraText) }
        return prompt(
            prompt = name,
            default = default?.also { muted("Previous ${name.lowercase()}: $it") }
        ) { input ->
            runBlocking {
                getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    override suspend fun getError(input: String): String? {
        return when {
            !validationRules.isRequired && input.isBlank() -> null
            input.isBlank() -> Errors.blankInput(name)
            validationRules.maxLength?.let { input.length > it } == true || validationRules.minLength?.let { input.length < it } == true -> {
                Errors.invalidLength(min = validationRules.minLength, max = validationRules.maxLength)
            }
            validationRules.pattern?.let { !input.matches(it) } == true -> Errors.invalidRegex(validationRules.pattern)
            else -> null
        }
    }
}
