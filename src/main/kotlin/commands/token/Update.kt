package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.option
import commands.prompt
import commands.success
import kotlinx.coroutines.runBlocking
import token.Token
import token.TokenStore
import utils.yesNoMenu

class Update : CliktCommand() {
    private val tokenStore = TokenStore()
    private val tokenParameter: String? by option("-t", "--token")

    override fun run(): Unit = runBlocking {
        if (tokenStore.token == null) {
            prompt(Token, tokenParameter).also { tokenStore.putToken(it) }
            success("Token set successfully")
        } else {
            val confirmed = when (tokenParameter) {
                null -> currentContext.terminal.yesNoMenu(default = true)
                else -> true
            }
            if (confirmed) {
                prompt(Token, tokenParameter).also { tokenStore.putToken(it) }
                success("Token changed successfully")
            } else {
                return@runBlocking
            }
        }
    }
}
