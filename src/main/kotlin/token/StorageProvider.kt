package token

import com.sun.jna.Platform
import token.windows.CredManagerBackedTokenStore

object StorageProvider {
    private var PERSISTED_TOKEN_STORE_CANDIDATES: List<SecretStore<TokenData>>

    init {
        val tokenStoreCandidates: MutableList<SecretStore<TokenData>> = mutableListOf()
        if (Platform.isWindows()) {
            tokenStoreCandidates.add(CredManagerBackedTokenStore())
        }
        PERSISTED_TOKEN_STORE_CANDIDATES = tokenStoreCandidates
    }

    fun getTokenStorage(): SecretStore<TokenData>? {
        return PERSISTED_TOKEN_STORE_CANDIDATES.firstOrNull()
    }
}
