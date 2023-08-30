package token

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.prompt
import io.ExitCode
import utils.Environment

object TokenStore {
    private const val CREDENTIAL_KEY = "komac/github-access-token"
    private val credentialStore = StorageProvider.getTokenStorage()
    private var storedToken = credentialStore?.get(CREDENTIAL_KEY)
    val token: String?
        get() = storedToken?.value

    fun putToken(tokenString: String, storeToken: Boolean = !Environment.isCI) {
        if (storeToken) credentialStore?.add(CREDENTIAL_KEY, TokenData(tokenString))
        storedToken = TokenData(tokenString)
    }

    fun deleteToken() = credentialStore?.delete(CREDENTIAL_KEY)

    suspend fun invalidTokenPrompt(terminal: Terminal) = with(terminal) {
        warning("Token is invalid. Please enter a new token.")
        val inputToken = prompt(Token, parameter = null, transform = { it }) ?: throw ProgramResult(ExitCode.CTRLC)
        putToken(inputToken)
        println()
    }
}
