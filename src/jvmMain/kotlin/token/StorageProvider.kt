package token

import token.windows.CredManagerBackedTokenStore
import utils.Platform

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
