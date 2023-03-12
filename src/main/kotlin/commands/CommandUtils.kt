package commands

import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.rendering.OverflowWrap
import com.github.ajalt.mordant.rendering.TextAlign
import com.github.ajalt.mordant.rendering.Whitespace
import com.github.ajalt.mordant.terminal.Terminal
import input.ExitCode

inline fun <reified T> Terminal.prompt(commandPrompt: CommandPrompt<T>, parameter: String? = null): T {
    val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    val error = commandPrompt.getError(parameter)
    return when {
        error != null -> {
            if (!isCIEnvironment) {
                danger(error)
                println()
                commandPrompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CtrlC.code)
            } else {
                throw CliktError(colors.danger(error), statusCode = 1)
            }
        }
        parameter != null && parameter is T -> parameter
        isCIEnvironment -> throw CliktError(
            message = colors.danger(
                buildString {
                    append(Errors.error)
                    append(" ")
                    append(commandPrompt::class.simpleName?.replace("([A-Z])".toRegex(), " $1")?.trim() ?: "Parameter")
                    append(" not provided")
                }
            ),
            statusCode = 1
        )
        else -> commandPrompt.prompt(this)?.also { println() } ?: throw ProgramResult(ExitCode.CtrlC.code)
    }
}

inline fun <reified T> CliktCommand.prompt(commandPrompt: CommandPrompt<T>, parameter: String? = null): T {
    return currentContext.terminal.prompt(commandPrompt, parameter)
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
