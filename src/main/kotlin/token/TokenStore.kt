package token

import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.microsoft.alm.secret.Token
import com.microsoft.alm.secret.TokenType
import com.microsoft.alm.storage.SecretStore
import com.microsoft.alm.storage.macosx.KeychainSecurityBackedTokenStore
import com.microsoft.alm.storage.posix.GnomeKeyringBackedTokenStore
import com.microsoft.alm.storage.windows.CredManagerBackedTokenStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import java.io.IOException

@Single
class TokenStore {
    private val credentialStore = getCredentialStore()
    var token: Deferred<String> = CoroutineScope(Dispatchers.IO).async { getToken(Terminal()) }

    private suspend fun getToken(terminal: Terminal): String {
        return if (credentialStore[credentialKey] == null) {
            promptForToken(terminal).also { putToken(it) }
        } else {
            val credentialToken = credentialStore[credentialKey].Value
            val (tokenValid, ioException) = checkIfTokenValid(credentialToken)
            if (tokenValid) {
                credentialToken
            } else {
                terminal.warning(ioException ?: "Token is invalid. Please enter a new token.")
                promptForToken(terminal).also { putToken(it) }
            }
        }
    }

    suspend fun putToken(tokenString: String) = coroutineScope {
        credentialStore.add(credentialKey, Token(tokenString, TokenType.Personal))
        token = async { tokenString }
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
        return terminal.prompt(
            prompt = terminal.colors.brightGreen("Please enter your GitHub personal access token"),
            convert = {
                val (tokenValid, ioException) = checkIfTokenValid(it)
                if (tokenValid) {
                    ConversionResult.Valid(it)
                } else {
                    ConversionResult.Invalid(ioException?.message ?: "Invalid token. Please try again.")
                }
            }
        )!!
    }

    private fun checkIfTokenValid(tokenString: String): Pair<Boolean, IOException?> {
        return try {
            GitHubBuilder().withOAuthToken(tokenString).build().isCredentialValid to null
        } catch (ioException: IOException) {
            false to ioException
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
