package token

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import com.microsoft.alm.secret.Token
import com.microsoft.alm.secret.TokenType
import com.microsoft.alm.storage.SecretStore
import com.microsoft.alm.storage.macosx.KeychainSecurityBackedTokenStore
import com.microsoft.alm.storage.posix.GnomeKeyringBackedTokenStore
import com.microsoft.alm.storage.windows.CredManagerBackedTokenStore
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import java.io.IOException

@Single
class TokenStore {
    lateinit var token: String
    fun getToken(terminal: Terminal) {
        val credentialStore = getCredentialStore()
        if (credentialStore[credentialKey] == null) {
            terminal.promptForToken().also {
                credentialStore.add(credentialKey, Token(it, TokenType.Personal))
                token = it
            }
        } else {
            token = credentialStore[credentialKey].Value
        }
    }

    private fun getCredentialStore(): SecretStore<Token> {
        val operatingSystem = System.getProperty(osName)
        return when {
            operatingSystem.startsWith(windows) -> CredManagerBackedTokenStore()
            operatingSystem.startsWith(linux) -> GnomeKeyringBackedTokenStore()
            operatingSystem.startsWith(mac) -> KeychainSecurityBackedTokenStore()
            else -> throw IOException("Unsupported operating system")
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
        private const val linux = "Linux"
        private const val mac = "Mac"
        private const val credentialUsername = "github-access-token"
        private const val credentialKey = "komac/$credentialUsername"
    }
}
