package token

import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandUtils.prompt
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.koin.core.annotation.Single

@Single
class TokenStore {
    private val credentialStore = StorageProvider.getTokenStorage()
    private var storedToken = credentialStore?.get(credentialKey)
    val token: String?
        get() = storedToken?.value
    var isTokenValid: Deferred<Boolean> = CoroutineScope(Dispatchers.IO).async { Token.isTokenValid(token) }

    suspend fun putToken(tokenString: String) = coroutineScope {
        credentialStore?.add(credentialKey, TokenData(tokenString))
        storedToken = TokenData(tokenString)
        isTokenValid = async { true }
    }

    suspend fun useTokenParameter(tokenString: String): Boolean {
        isTokenValid = coroutineScope { async { Token.isTokenValid(tokenString) } }
        storedToken = TokenData(tokenString)
        return isTokenValid.await()
    }

    fun deleteToken() = credentialStore?.delete(credentialKey)

    suspend fun invalidTokenPrompt(terminal: Terminal) = with(terminal) {
        warning("Token is invalid. Please enter a new token.")
        prompt(Token).also { putToken(it) }
    }

    companion object {
        private const val credentialKey = "komac/github-access-token"
    }
}
