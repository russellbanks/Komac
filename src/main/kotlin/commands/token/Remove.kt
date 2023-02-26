package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import token.TokenStore
import utils.yesNoMenu

class Remove : CliktCommand(), KoinComponent {
    private val tokenStore: TokenStore by inject()
    private val skipPrompt: Boolean by option("-y", "--yes").flag(default = false)

    override fun run() = runBlocking {
        with(currentContext.terminal) {
            val shouldDeleteToken = if (skipPrompt) {
                true
            } else {
                warning("Would you like to remove the currently stored token?")
                yesNoMenu(default = false)
            }
            if (shouldDeleteToken) {
                tokenStore.deleteToken()
                success("The token has successfully been removed")
            } else {
                info("The token has not been removed")
            }
        }
    }
}
