package token

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import com.microsoft.alm.secret.Token
import com.microsoft.alm.secret.TokenType
import com.microsoft.alm.storage.windows.CredManagerBackedTokenStore
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import java.io.IOException

@Single
class TokenStore {
    lateinit var token: String
    fun getToken(terminal: Terminal) {
        val operatingSystem = System.getProperty(osName)
        if (operatingSystem.startsWith(windows)) {
            val credentialStore = CredManagerBackedTokenStore()
            if (credentialStore[credentialKey] == null) {
                terminal.promptForToken().also {
                    credentialStore.add(credentialKey, Token(it, TokenType.Personal))
                    token = it
                }
            } else {
                token = credentialStore[credentialKey].Value
            }
        }
    }

    private fun Terminal.promptForToken(): String {
        var token: String?
        do {
            println(brightGreen("Please enter your GitHub personal access token:"))
            token = prompt(brightWhite("Token"))
            val isValid = try {
                GitHubBuilder().withOAuthToken(token).build()
                true
            } catch (_: IOException) {
                println(red("Invalid token. Please try again."))
                false
            }
        } while (token == null || !isValid)
        return token
    }

    companion object {
        private const val osName = "os.name"
        private const val windows = "Windows"
        private const val credentialUsername = "github-access-token"
        private const val credentialKey = "komac/$credentialUsername"
    }
}
