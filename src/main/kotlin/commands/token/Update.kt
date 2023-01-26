package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import token.TokenStore

class Update : CliktCommand(), KoinComponent {
    private val tokenStore: TokenStore by inject()

    override fun run() = runBlocking {
        if (tokenStore.token == null) {
            tokenStore.promptForToken(currentContext.terminal)
        } else {
            val confirmed = confirm(text = "Would you like to change the currently stored token?", default = true)
            if (confirmed == true) {
                tokenStore.promptForToken(currentContext.terminal).also { tokenStore.putToken(it) }
                currentContext.terminal.success("Token changed successfully")
            } else {
                return@runBlocking
            }
        }
    }
}
