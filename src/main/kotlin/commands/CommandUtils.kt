package commands

import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.rendering.OverflowWrap
import com.github.ajalt.mordant.rendering.TextAlign
import com.github.ajalt.mordant.rendering.Whitespace
import com.github.ajalt.mordant.terminal.Terminal
import commands.interfaces.CheckMenuPrompt
import commands.interfaces.ListPrompt
import commands.interfaces.Prompt
import commands.interfaces.RadioMenuPrompt
import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import input.ExitCode
import io.ktor.http.Url

suspend fun <T> Terminal.prompt(prompt: Prompt<T>, parameter: String? = null, transform: (String) -> T): T {
    val isCIEnvironmentPresent = System.getenv("CI")?.toBooleanStrictOrNull() == true
    val error = parameter?.let { prompt.getError(it) }
    return when {
        error != null -> {
            if (!isCIEnvironmentPresent) {
                danger(error)
                println()
                prompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CtrlC)
            } else {
                throw CliktError(colors.danger(error), statusCode = 1)
            }
        }
        parameter != null -> transform(parameter)
        isCIEnvironmentPresent -> throw CliktError(
            message = colors.danger(
                buildString {
                    append(Errors.error)
                    append(" ")
                    append(prompt::class.simpleName?.replace("([A-Z])".toRegex(), " $1")?.trim() ?: "Parameter")
                    append(" not provided")
                }
            ),
            statusCode = 1
        )
        else -> prompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CtrlC)
    }
}

suspend fun CliktCommand.prompt(textPrompt: TextPrompt, parameter: String? = null): String {
    return currentContext.terminal.prompt(textPrompt, parameter, transform = { it })
}

suspend fun <T> CliktCommand.prompt(listPrompt: ListPrompt<T>, parameter: String? = null): List<T> {
    return currentContext.terminal.prompt(listPrompt, parameter, transform = { listPrompt.validationRules.transform(it) })
}

suspend fun CliktCommand.prompt(urlPrompt: UrlPrompt, parameter: String? = null): Url {
    return currentContext.terminal.prompt(urlPrompt, parameter, transform = { urlPrompt.transform(it) })
}

suspend fun <T> CliktCommand.prompt(radioMenuPrompt: RadioMenuPrompt<T>, parameter: String? = null): T? {
    return currentContext.terminal.prompt(radioMenuPrompt, parameter, transform = { it as T })
}

suspend fun <T> CliktCommand.prompt(checkMenuPrompt: CheckMenuPrompt<T>, parameter: String? = null): List<T>? {
    return currentContext.terminal.prompt(checkMenuPrompt, parameter, transform = { it as List<T> })
}

fun CliktCommand.info(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = currentContext.terminal.info(message, whitespace, align, overflowWrap, width)

fun CliktCommand.success(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = currentContext.terminal.success(message, whitespace, align, overflowWrap, width)

fun CliktCommand.warning(
    message: Any?,
    whitespace: Whitespace = Whitespace.PRE,
    align: TextAlign = TextAlign.NONE,
    overflowWrap: OverflowWrap = OverflowWrap.NORMAL,
    width: Int? = null
) = currentContext.terminal.warning(message, whitespace, align, overflowWrap, width)

val CliktCommand.colors get() = currentContext.terminal.colors
