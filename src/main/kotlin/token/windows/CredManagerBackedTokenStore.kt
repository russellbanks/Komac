package token.windows

import token.Token

class CredManagerBackedTokenStore : CredManagerBackedSecureStore<Token>() {
    override fun create(username: String, secret: String): Token {
        return Token(secret)
    }

    override fun getUsername(secret: Token): String = TOKEN_USERNAME

    override fun getCredentialBlob(secret: Token): String = secret.value

    companion object {
        const val TOKEN_USERNAME = "PersonalAccessToken"
    }
}
