package commands

import Environment
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.core.terminal
import com.github.ajalt.mordant.rendering.OverflowWrap
import com.github.ajalt.mordant.rendering.TextAlign
import com.github.ajalt.mordant.rendering.Whitespace
import com.github.ajalt.mordant.terminal.Terminal
import io.ExitCode
import io.ktor.http.Url
import io.menu.prompts.CheckMenuPrompt
import io.menu.prompts.ListPrompt
import io.menu.prompts.Prompt
import io.menu.prompts.RadioMenuPrompt
import io.menu.prompts.TextPrompt
import io.menu.prompts.UrlPrompt

suspend fun <T> Terminal.prompt(prompt: Prompt<T>, parameter: String? = null, transform: (String) -> T): T {
    val error = parameter?.let { prompt.getError(it) }
    return when {
        error != null -> if (!Environment.isCI) {
            danger(error)
            println()
            prompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CTRLC)
        } else {
            throw CliktError(theme.danger(error), statusCode = 1)
        }
        parameter != null -> transform(parameter)
        Environment.isCI -> throw CliktError(
            message = theme.danger("${prompt.name} was not provided"),
            statusCode = 1
        )
        else -> prompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CTRLC)
    }
}

suspend fun CliktCommand.prompt(textPrompt: TextPrompt, parameter: String? = null): String {
    return terminal.prompt(textPrompt, parameter, transform = { it })
}

suspend fun <T> CliktCommand.prompt(listPrompt: ListPrompt<T>, parameter: String? = null): List<T> {
    return terminal.prompt(
        listPrompt,
        parameter,
        transform = { listPrompt.validationRules.transform(it) }
    )
}

suspend fun CliktCommand.prompt(urlPrompt: UrlPrompt, parameter: String? = null): Url {
    return terminal.prompt(urlPrompt, parameter, transform = { urlPrompt.validationRules.transform(it) })
}

suspend fun <T> CliktCommand.prompt(radioMenuPrompt: RadioMenuPrompt<T>, parameter: String? = null): T {
    return terminal.prompt(radioMenuPrompt, parameter, transform = { it as T })
}

suspend fun <T> CliktCommand.prompt(checkMenuPrompt: CheckMenuPrompt<T>, parameter: String? = null): List<T> {
    return terminal.prompt(checkMenuPrompt, parameter, transform = { it as List<T> })
}

fun CliktCommand.info(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = terminal.info(message, whitespace, align, overflowWrap, width)

fun CliktCommand.success(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = terminal.success(message, whitespace, align, overflowWrap, width)

fun CliktCommand.warning(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = terminal.warning(message, whitespace, align, overflowWrap, width)

val CliktCommand.theme get() = terminal.theme
