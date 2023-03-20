package commands.interfaces

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import input.Prompts
import kotlinx.coroutines.runBlocking

interface ListPrompt<T> : Prompt<List<T>> {
    val name: String

    val validationRules: ListValidationRules<T>

    val default: List<T>? get() = null

    val extraText: String?

    val description: String

    override suspend fun prompt(terminal: Terminal): List<T> = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Enter the $name (Max 25)"))
        if (extraText != null) info(extraText)
        return prompt(
            prompt = name,
            default = default?.also { muted("Previous ${name.lowercase()}: $it") }
        ) { input ->
            runBlocking {
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(validationRules.transform(input))
            }
        } ?: throw ProgramResult(0)
    }

    override suspend fun getError(input: String): String? {
        val transformedInput = validationRules.transform(input)
        val items = transformedInput.map { it.toString() }
        return when {
            transformedInput.size > validationRules.maxItems -> Errors.invalidLength(max = validationRules.maxItems)
            items.any { it.length > validationRules.maxItemLength ?: Int.MAX_VALUE } -> {
                Errors.invalidLength(
                    min = validationRules.minItemLength,
                    max = validationRules.maxItemLength,
                    items = items.filter { it.length > validationRules.maxItemLength as Int }
                )
            }
            validationRules.regex?.let { regex -> items.any { !it.matches(regex) } } == true -> {
                Errors.invalidRegex(
                    regex = validationRules.regex,
                    items = items.filterNot { it matches validationRules.regex as Regex }
                )
            }
            validationRules.additionalValidation?.invoke(transformedInput) != null -> {
                validationRules.additionalValidation?.invoke(transformedInput)
            }
            else -> null
        }
    }
}
