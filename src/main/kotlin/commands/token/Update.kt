package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.option
import commands.CommandUtils.prompt
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import token.Token
import token.TokenStore

class Update : CliktCommand(), KoinComponent {
    private val tokenStore: TokenStore by inject()
    private val tokenParameter: String? by option("-t", "--token")

    override fun run(): Unit = runBlocking {
        with(currentContext.terminal) {
            if (tokenStore.token == null) {
                prompt(Token, tokenParameter).also { tokenStore.putToken(it) }
                success("Token set successfully")
            } else {
                val confirmed = when (tokenParameter) {
                    null -> confirm(text = "Would you like to change the currently stored token?", default = true)
                    else -> true
                }
                if (confirmed == true) {
                    prompt(Token, tokenParameter).also { tokenStore.putToken(it) }
                    success("Token changed successfully")
                } else {
                    return@runBlocking
                }
            }
        }
    }
}
