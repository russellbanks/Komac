package token

import com.sun.jna.Platform
import token.windows.CredManagerBackedTokenStore

object StorageProvider {
    private var PERSISTED_TOKEN_STORE_CANDIDATES: List<SecretStore<Token>>

    init {
        val tokenStoreCandidates: MutableList<SecretStore<Token>> = mutableListOf()
        if (Platform.isWindows()) {
            tokenStoreCandidates.add(CredManagerBackedTokenStore())
        }
        PERSISTED_TOKEN_STORE_CANDIDATES = tokenStoreCandidates
    }

    fun getTokenStorage(): SecretStore<Token>? {
        return PERSISTED_TOKEN_STORE_CANDIDATES.firstOrNull()
    }
}
