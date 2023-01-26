package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import token.TokenStore

class Remove : CliktCommand(), KoinComponent {
    private val tokenStore: TokenStore by inject()

    override fun run() = runBlocking {
        val shouldDelete = confirm(
            text = currentContext.terminal.colors.warning("Would you like to remove the currently stored token?")
        )
        if (shouldDelete == true) {
            tokenStore.deleteToken()
            currentContext.terminal.success("The token has successfully been removed")
        } else {
            currentContext.terminal.info("The token has not been removed")
        }
    }
}
