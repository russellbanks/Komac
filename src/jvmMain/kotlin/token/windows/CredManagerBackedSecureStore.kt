package token.windows

import com.sun.jna.LastErrorException
import com.sun.jna.Memory
import com.sun.jna.Platform
import com.sun.jna.Pointer
import okio.ByteString
import okio.ByteString.Companion.encodeUtf8
import okio.ByteString.Companion.toByteString
import token.SecretStore
import token.TokenData

/**
 * This class exposes functions to interact with Windows Credential Manager
 */
abstract class CredManagerBackedSecureStore<E : TokenData?> : SecretStore<E> {
    private val instance = credAdvapi32Instance

    /**
     * Create a `Secret` from the string representation
     *
     * @param username
     * username for the secret
     * @param secret
     * password, oauth2 access token, or Personal Access Token
     *
     * @return a `Secret` from the input
     */
    protected abstract fun create(username: String, secret: String): E

    /**
     * Get String representation of the UserName field from the `Secret`
     *
     * @param secret
     * A `Credential`, `Token` or `TokenPair`
     *
     * @return username from this secret
     */
    protected abstract fun getUsername(secret: E): String

    /**
     * Get String representation of the CredentialBlob field from the secret
     *
     * @param secret
     * A `Credential`, `Token` or `TokenPair`
     *
     * @return credential from this secret
     */
    protected abstract fun getCredentialBlob(secret: E): String

    /**
     * Read calls CredRead on Windows and retrieve the Secret
     *
     * Multi-thread safe, synchronized access to store
     *
     * @param key
     * TargetName in the credential structure
     */
    override fun get(key: String): E? {
        val pcredential = CredAdvapi32.PCREDENTIAL()
        var read: Boolean
        var cred: E?
        try {
            // MSDN doc doesn't mention threading safety, so let's just be careful and synchronize the access
            synchronized(instance) { read = instance.CredRead(key, CredAdvapi32.CRED_TYPE_GENERIC, 0, pcredential) }
            cred = if (read) {
                val credential = pcredential.credential?.let { CredAdvapi32.CREDENTIAL(it) }
                val secretBytes: ByteString? = credential?.CredentialBlob
                    ?.getByteArray(0, credential.CredentialBlobSize)
                    ?.toByteString()
                val secret = secretBytes?.utf8()
                val username = credential?.UserName
                if (username != null && secret != null) create(username, secret) else null
            } else {
                null
            }
        } catch (_: LastErrorException) {
            cred = null
        } finally {
            synchronized(instance) { instance.CredFree(pcredential.credential) }
        }
        return cred
    }

    /**
     * Delete the stored credential from Credential Manager
     *
     * Multi-thread safe, synchronized access to store
     *
     * @param key
     * TargetName in the credential structure
     *
     * @return
     * true if delete successful, false otherwise (including key doesn't exist)
     */
    override fun delete(key: String): Boolean {
        try {
            synchronized(instance) { return instance.CredDelete(key, CredAdvapi32.CRED_TYPE_GENERIC, 0) }
        } catch (_: LastErrorException) {
            return false
        }
    }

    /**
     * Add the specified secret to Windows Credential Manager
     *
     * Multi-thread safe, synchronized access to store
     * @param key
     * TargetName in the credential structure
     * @param secret
     * secret to be stored
     *
     * @return `true` if successfully added
     * `false` otherwise
     */
    override fun add(key: String, secret: E): Boolean {
        val username = getUsername(secret)
        val credentialBlob = getCredentialBlob(secret)
        val credBlob = credentialBlob.encodeUtf8()
        val cred = buildCred(key, username, credBlob)
        return try {
            synchronized(instance) { instance.CredWrite(cred, 0) }
            true
        } catch (_: LastErrorException) {
            false
        } finally {
            cred.CredentialBlob?.clear(credBlob.size.toLong())
        }
    }

    private fun buildCred(key: String, username: String, credentialBlob: ByteString): CredAdvapi32.CREDENTIAL {
        val credential = CredAdvapi32.CREDENTIAL().apply {
            Flags = 0
            Type = CredAdvapi32.CRED_TYPE_GENERIC
            TargetName = key
            CredentialBlobSize = credentialBlob.size
            CredentialBlob = getPointer(credentialBlob)
            Persist = CredAdvapi32.CRED_PERSIST_LOCAL_MACHINE
            UserName = username
        }
        return credential
    }

    private fun getPointer(array: ByteString): Pointer {
        return Memory(array.size.toLong()).apply { write(0, array.toByteArray(), 0, array.size) }
    }

    companion object {
        private val credAdvapi32Instance: CredAdvapi32
            get() = if (Platform.isWindows()) {
                CredAdvapi32.INSTANCE
            } else {

                // Return a dummy on other platforms
                object : CredAdvapi32 {
                    @Throws(LastErrorException::class)
                    override fun CredRead(
                        targetName: String?,
                        type: Int,
                        flags: Int,
                        pcredential: CredAdvapi32.PCREDENTIAL?
                    ) = false

                    @Throws(LastErrorException::class)
                    override fun CredWrite(credential: CredAdvapi32.CREDENTIAL?, flags: Int) = false

                    @Throws(LastErrorException::class)
                    override fun CredDelete(targetName: String?, type: Int, flags: Int) = false
                    override fun CredFree(credential: Pointer?) = Unit
                }
            }
    }
}
