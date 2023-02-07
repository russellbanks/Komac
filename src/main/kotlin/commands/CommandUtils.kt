package commands

import Errors
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.Terminal

object CommandUtils {
    suspend inline fun <reified T> Terminal.prompt(commandPrompt: CommandPrompt<T>, parameter: String? = null): T {
        val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
        val error = commandPrompt.getError(parameter)
        return when {
            error != null -> {
                if (!isCIEnvironment) {
                    danger(error)
                    println()
                    commandPrompt.prompt(this)
                } else {
                    throw CliktError(colors.danger(error), statusCode = 1)
                }
            }
            parameter != null && parameter is T -> parameter
            isCIEnvironment -> throw CliktError(
                message = colors.danger("${Errors.error} Parameter not provided"),
                statusCode = 1
            )
            else -> commandPrompt.prompt(this)
        }
    }
}
