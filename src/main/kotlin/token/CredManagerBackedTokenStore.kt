package token

import com.microsoft.alm.secret.TokenType
import com.microsoft.alm.storage.windows.internal.CredManagerBackedSecureStore

abstract class CredManagerBackedTokenStore : CredManagerBackedSecureStore<Token?>() {
    override fun create(username: String?, secret: String): Token {
        return Token(secret, TokenType.Personal)
    }

    override fun getUsername(secret: Token?): String? = TOKEN_USERNAME

    override fun getCredentialBlob(secret: Token?): String? = secret?.Value

    companion object {
        const val TOKEN_USERNAME = "PersonalAccessToken"
    }
}