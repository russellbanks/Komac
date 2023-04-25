package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.option
import commands.prompt
import commands.success
import kotlinx.coroutines.runBlocking
import token.Token
import token.TokenStore
import utils.menu.yesNoMenu

class Update : CliktCommand() {
    private val tokenParameter: String? by option("-t", "--token")

    override fun run(): Unit = runBlocking {
        if (TokenStore.token == null) {
            prompt(Token, tokenParameter).also { TokenStore.putToken(it) }
            success("Token set successfully")
        } else {
            val confirmed = if (tokenParameter == null) {
                currentContext.terminal.yesNoMenu(default = true).prompt()
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
