package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.terminal
import com.github.ajalt.clikt.parameters.options.option
import commands.prompt
import commands.success
import io.menu.yesNoMenu
import kotlinx.coroutines.runBlocking
import token.Token
import token.TokenStore

class Update : CliktCommand() {
    private val tokenParameter: String? by option("-t", "--token", help = "The new token to use")

    override fun run(): Unit = runBlocking {
        if (TokenStore.token == null) {
            prompt(Token, tokenParameter).also { TokenStore.putToken(it) }
            success("Token set successfully")
        } else {
            val confirmed = if (tokenParameter == null) {
                terminal.yesNoMenu(default = true).prompt()
            } else {
                true
            }
            if (confirmed) {
                prompt(Token, tokenParameter).also { TokenStore.putToken(it) }
                success("Token changed successfully")
            } else {
                return@runBlocking
            }
        }
    }
}
