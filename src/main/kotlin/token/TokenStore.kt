package token

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
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
            promptForToken(terminal).also { putToken(it, credentialStore) }
        } else {
            val credentialToken = credentialStore[credentialKey].Value
            if (checkIfTokenValid(credentialToken)) {
                token = credentialToken
            } else {
                terminal.println(brightRed("Token is invalid. Please enter a new token."))
                promptForToken(terminal).also { putToken(it, credentialStore) }
            }
        }
    }

    fun putToken(tokenString: String, credentialStore: SecretStore<Token> = getCredentialStore()) {
        credentialStore.add(credentialKey, Token(tokenString, TokenType.Personal))
        token = tokenString
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

    fun promptForToken(terminal: Terminal): String {
        var token: String?
        do {
            println(brightGreen("Please enter your GitHub personal access token:"))
            token = terminal.prompt(brightWhite("Token"))
            val isValid = token?.let { checkIfTokenValid(it) }
        } while (token == null || isValid == false)
        return token
    }

    private fun checkIfTokenValid(tokenString: String): Boolean {
        return try {
            GitHubBuilder().withOAuthToken(tokenString).build().isCredentialValid
        } catch (ioException: IOException) {
            println(brightRed(ioException.message ?: "Invalid token. Please try again."))
            false
        }
    }

    companion object {
        private const val osName = "os.name"
        private const val windows = "Windows"
        private const val linux = "Linux"
        private const val mac = "Mac"
        private const val credentialKey = "komac/github-access-token"
    }
}
