package token.windows

import token.TokenData

class CredManagerBackedTokenStore : CredManagerBackedSecureStore<TokenData>() {
    override fun create(username: String, secret: String): TokenData {
        return TokenData(secret)
    }

    override fun getUsername(secret: TokenData): String = TOKEN_USERNAME

    override fun getCredentialBlob(secret: TokenData): String = secret.value

    companion object {
        const val TOKEN_USERNAME = "PersonalAccessToken"
    }
}
