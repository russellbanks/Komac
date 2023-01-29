package token

import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.kohsuke.github.GitHub
import org.koin.core.annotation.Single
import java.io.IOException
import kotlin.system.exitProcess

@Single
class TokenStore {
    private val credentialStore = StorageProvider.getTokenStorage()
        ?: throw UnsupportedOperationException("Could not find secure token storage for the current operating system")
    private var storedToken = credentialStore[credentialKey]
    val token: String?
        get() = storedToken?.value
    var isTokenValid: Deferred<Boolean> = CoroutineScope(Dispatchers.IO).async { checkIfTokenValid(token) }

    suspend fun putToken(tokenString: String) = coroutineScope {
        credentialStore.add(credentialKey, Token(tokenString))
        storedToken = Token(tokenString)
        isTokenValid = async { true }
    }

    suspend fun useTokenParameter(tokenString: String): Boolean {
        isTokenValid = coroutineScope { async { checkIfTokenValid(tokenString) } }
        storedToken = Token(tokenString)
        return isTokenValid.await()
    }

    fun deleteToken() = credentialStore.delete(credentialKey)

    suspend fun promptForToken(terminal: Terminal): String {
        return terminal.prompt(
            prompt = terminal.colors.brightGreen("Please enter your GitHub personal access token"),
            convert = {
                if (checkIfTokenValid(it)) {
                    ConversionResult.Valid(it)
                } else {
                    ConversionResult.Invalid("Invalid token. Please try again.")
                }
            }
        )?.also { putToken(it) } ?: exitProcess(ExitCode.CtrlC.code)
    }

    private fun checkIfTokenValid(tokenString: String?): Boolean {
        return try {
            GitHub.connectUsingOAuth(tokenString).isCredentialValid
        } catch (_: IOException) {
            false
        }
    }

    suspend fun invalidTokenPrompt(terminal: Terminal) {
        terminal.warning("Token is invalid. Please enter a new token.")
        promptForToken(terminal).also { putToken(it) }
    }

    companion object {
        private const val credentialKey = "komac/github-access-token"
    }
}
