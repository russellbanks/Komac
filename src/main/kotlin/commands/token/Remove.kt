package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import token.TokenStore

class Remove : CliktCommand(), KoinComponent {
    private val tokenStore: TokenStore by inject()
    private val skipPrompt: Boolean by option("-y", "--yes").flag(default = false)

    override fun run() = runBlocking {
        val shouldDelete = if (skipPrompt) {
            true
        } else {
            confirm(
                text = currentContext.terminal.colors.warning("Would you like to remove the currently stored token?")
            )
        }
        if (shouldDelete == true) {
            tokenStore.deleteToken()
            currentContext.terminal.success("The token has successfully been removed")
        } else {
            currentContext.terminal.info("The token has not been removed")
        }
    }
}
