package token

import com.microsoft.alm.secret.Secret
import com.microsoft.alm.secret.Token
import com.microsoft.alm.storage.InsecureFileBackedTokenStore
import com.microsoft.alm.storage.InsecureInMemoryStore
import com.microsoft.alm.storage.SecretStore
import com.microsoft.alm.storage.macosx.KeychainSecurityBackedTokenStore
import com.microsoft.alm.storage.posix.GnomeKeyringBackedTokenStore
import com.microsoft.alm.storage.posix.internal.GnomeKeyringBackedSecureStore
import com.microsoft.alm.storage.windows.CredManagerBackedTokenStore
import com.sun.jna.Platform

object StorageProvider {
    private var PERSISTED_TOKEN_STORE_CANDIDATES: List<SecretStore<Token>>? = null

    init {
        val tokenStoreCandidates: MutableList<SecretStore<Token>> = ArrayList()
        if (Platform.isWindows()) {
            tokenStoreCandidates.add(CredManagerBackedTokenStore())
        }
        if (Platform.isMac()) {
            tokenStoreCandidates.add(KeychainSecurityBackedTokenStore())
        }
        if (Platform.isLinux() && GnomeKeyringBackedSecureStore.isGnomeKeyringSupported()) {
            tokenStoreCandidates.add(GnomeKeyringBackedTokenStore())
        }
        tokenStoreCandidates.add(InsecureFileBackedTokenStore())
        PERSISTED_TOKEN_STORE_CANDIDATES = tokenStoreCandidates
    }

    fun getTokenStorage(persist: Boolean, secureOption: SecureOption): SecretStore<Token>? {
        val inMemoryStoreGenerator: NonPersistentStoreGenerator<Token> = object : NonPersistentStoreGenerator<Token> {
            override val insecureNonPersistentStore: SecretStore<Token>
                get() = InsecureInMemoryStore()
            override val secureNonPersistentStore: SecretStore<Token>?
                get() {
                    return null
                }
        }
        return getStore(persist, secureOption, PERSISTED_TOKEN_STORE_CANDIDATES!!, inMemoryStoreGenerator)
    }

    private fun <E : Secret?> findSecureStore(stores: List<SecretStore<E>>?): SecretStore<E>? {
        if (stores != null) {
            for (store in stores) {
                if (store.isSecure) {
                    return store
                }
            }
        }
        return null
    }

    private fun <E : Secret?> findPersistedStore(
        secureOption: SecureOption?,
        stores: List<SecretStore<E>>?
    ): SecretStore<E>? {
        var candidate: SecretStore<E>? = findSecureStore(stores)
        if (candidate == null && secureOption == SecureOption.PREFER) {
            // just return any store from the list since none of them is secure
            if (stores!!.isNotEmpty()) {
                candidate = stores[0]
            }
        }

        return candidate
    }

    fun <E : Secret?> getStore(
        persist: Boolean,
        secureOption: SecureOption?,
        stores: List<SecretStore<E>>,
        nonPersistentStoreGenerator: NonPersistentStoreGenerator<E>
    ): SecretStore<E>? {
        var candidate: SecretStore<E>?
        if (persist) {
            candidate = findPersistedStore(secureOption, stores)
        } else {
            // not persisted
            candidate = nonPersistentStoreGenerator.secureNonPersistentStore
            if (candidate == null && secureOption == SecureOption.PREFER) {
                candidate = nonPersistentStoreGenerator.insecureNonPersistentStore
            }
        }
        return candidate
    }

    enum class SecureOption {
        /**
         * The store must be secure, i.e. generally the storage needs to be password protected
         * and data potentially is encrypted
         *
         * However, this program makes no assertion on *how* secure the storage really is.  It's only
         * an attribute on the storage
         */
        MUST,

        /**
         * Prefer a secure storage, but if none is available, a unprotected, non-secure storage will be returned
         */
        PREFER
    }

    interface NonPersistentStoreGenerator<E : Secret?> {
        val insecureNonPersistentStore: SecretStore<E>
        val secureNonPersistentStore: SecretStore<E>?
    }
}