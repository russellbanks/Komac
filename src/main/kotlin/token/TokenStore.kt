package token

import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.microsoft.alm.secret.Token
import com.microsoft.alm.secret.TokenType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import java.io.IOException
import kotlin.system.exitProcess

@Single
class TokenStore {
    private val credentialStore = StorageProvider.getTokenStorage(
        /* persist = */ true,
        /* secureOption = */ StorageProvider.SecureOption.MUST
    ) ?: throw UnsupportedOperationException("Could not find secure token storage for the current operating system")
    var storedToken: Deferred<String> = CoroutineScope(Dispatchers.IO).async { getToken(Terminal()) }

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
        storedToken = async { tokenString }
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
        ) ?: exitProcess(ExitCode.CtrlC.code)
    }

    private fun checkIfTokenValid(tokenString: String): Pair<Boolean, IOException?> {
        return try {
            GitHubBuilder().withOAuthToken(tokenString).build().isCredentialValid to null
        } catch (ioException: IOException) {
            false to ioException
        }
    }

    companion object {
        private const val credentialKey = "komac/github-access-token"
    }
}
