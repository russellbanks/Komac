package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.terminal
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import commands.info
import commands.success
import commands.warning
import io.menu.yesNoMenu
import kotlinx.coroutines.runBlocking
import token.TokenStore

class Remove : CliktCommand() {
    private val skipPrompt: Boolean by option(
        "-y", "--yes",
        help = "Skips the warning prompt"
    ).flag(default = false)

    override fun run() = runBlocking {
        val shouldDeleteToken = if (skipPrompt) {
            true
        } else {
            warning("Would you like to remove the currently stored token?")
            terminal.yesNoMenu(default = false).prompt()
        }
        if (shouldDeleteToken) {
            TokenStore.deleteToken()
            success("The token has successfully been removed")
        } else {
            info("The token has not been removed")
        }
    }
}
