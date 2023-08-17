package token

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.prompt
import io.ExitCode
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope

object TokenStore {
    private const val CREDENTIAL_KEY = "komac/github-access-token"
    private val credentialStore = StorageProvider.getTokenStorage()
    private var storedToken = credentialStore?.get(CREDENTIAL_KEY)
    val token: String?
        get() = storedToken?.value
    var isTokenValid: Deferred<Boolean> = CoroutineScope(Dispatchers.IO).async { Token.isTokenValid(token) }

    suspend fun putToken(tokenString: String) = coroutineScope {
        credentialStore?.add(CREDENTIAL_KEY, TokenData(tokenString))
        storedToken = TokenData(tokenString)
        isTokenValid = async { true }
    }

    suspend fun useTokenParameter(tokenString: String): Boolean {
        isTokenValid = coroutineScope { async { Token.isTokenValid(tokenString) } }
        storedToken = TokenData(tokenString)
        return isTokenValid.await()
    }

    fun deleteToken() = credentialStore?.delete(CREDENTIAL_KEY)

    suspend fun invalidTokenPrompt(terminal: Terminal) = with(terminal) {
        warning("Token is invalid. Please enter a new token.")
        val inputToken = prompt(Token, parameter = null, transform = { it }) ?: throw ProgramResult(ExitCode.CTRLC)
        putToken(inputToken)
        println()
    }
}
