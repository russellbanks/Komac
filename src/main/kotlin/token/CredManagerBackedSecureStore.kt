package token

import com.microsoft.alm.helpers.Debug
import com.microsoft.alm.helpers.LoggingHelper
import com.microsoft.alm.helpers.StringHelper
import com.microsoft.alm.helpers.SystemHelper
import com.microsoft.alm.secret.Secret
import com.microsoft.alm.storage.SecretStore
import com.sun.jna.LastErrorException
import com.sun.jna.Memory
import com.sun.jna.Pointer
import org.slf4j.LoggerFactory
import java.util.Arrays

/**
 * This class exposes functions to interact with Windows Credential Manager
 */
abstract class CredManagerBackedSecureStore<E : Secret?> : SecretStore<E?> {
    private val INSTANCE: CredAdvapi32 = credAdvapi32Instance

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
    protected abstract fun create(username: String?, secret: String?): E

    /**
     * Get String representation of the UserName field from the `Secret`
     *
     * @param secret
     * A `Credential`, `Token` or `TokenPair`
     *
     * @return username from this secret
     */
    protected abstract fun getUsername(secret: E?): String

    /**
     * Get String representation of the CredentialBlob field from the secret
     *
     * @param secret
     * A `Credential`, `Token` or `TokenPair`
     *
     * @return credential from this secre
     */
    protected abstract fun getCredentialBlob(secret: E?): String

    /**
     * Read calls CredRead on Windows and retrieve the Secret
     *
     * Multi-thread safe, synchronized access to store
     *
     * @param key
     * TargetName in the credential structure
     */
    override fun get(key: String): E? {
        logger.info("Getting secret for {}", key)
        val pcredential = CredAdvapi32.PCREDENTIAL()
        var read = false
        var cred: E?
        try {
            // MSDN doc doesn't mention threading safety, so let's just be careful and synchronize the access
            synchronized(INSTANCE) { read = INSTANCE.CredRead(key, CredAdvapi32.CRED_TYPE_GENERIC, 0, pcredential) }
            cred = if (read) {
                val credential = CredAdvapi32.CREDENTIAL(pcredential.credential)
                val secretBytes: ByteArray? = credential.CredentialBlob?.getByteArray(0, credential.CredentialBlobSize)
                val secret = StringHelper.UTF8GetString(secretBytes)
                val username: String? = credential.UserName
                create(username, secret)
            } else {
                null
            }
        } catch (e: LastErrorException) {
            LoggingHelper.logError(logger, "Getting secret failed.", e)
            cred = null
        } finally {
            if (pcredential.credential != null) {
                synchronized(INSTANCE) { INSTANCE.CredFree(pcredential.credential) }
            }
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
        logger.info("Deleting secret for {}", key)
        try {
            synchronized(INSTANCE) { return INSTANCE.CredDelete(key, CredAdvapi32.CRED_TYPE_GENERIC, 0) }
        } catch (e: LastErrorException) {
            LoggingHelper.logError(logger, "Deleting secret failed.", e)
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
    override fun add(key: String, secret: E?): Boolean {
        Debug.Assert(secret != null, "Secret cannot be null")
        logger.info("Adding secret for {}", key)
        val username = getUsername(secret)
        val credentialBlob = getCredentialBlob(secret)
        val credBlob = StringHelper.UTF8GetBytes(credentialBlob)
        val cred: CredAdvapi32.CREDENTIAL = buildCred(key, username, credBlob)
        return try {
            synchronized(INSTANCE) { INSTANCE.CredWrite(cred, 0) }
            true
        } catch (e: LastErrorException) {
            LoggingHelper.logError(logger, "Adding secret failed.", e)
            false
        } finally {
            cred.CredentialBlob?.clear(credBlob.size.toLong())
            Arrays.fill(credBlob, 0.toByte())
        }
    }

    /**
     * Windows credential manager is considered a secure storage for secrets
     *
     * @return `true` for Windows Credential Manager
     */
    override fun isSecure(): Boolean {
        return true
    }

    private fun buildCred(key: String, username: String, credentialBlob: ByteArray): CredAdvapi32.CREDENTIAL {
        val credential = CredAdvapi32.CREDENTIAL()
        credential.Flags = 0
        credential.Type = CredAdvapi32.CRED_TYPE_GENERIC
        credential.TargetName = key
        credential.CredentialBlobSize = credentialBlob.size
        credential.CredentialBlob = getPointer(credentialBlob)
        credential.Persist = CredAdvapi32.CRED_PERSIST_LOCAL_MACHINE
        credential.UserName = username
        return credential
    }

    private fun getPointer(array: ByteArray): Pointer {
        val p: Pointer = Memory(array.size.toLong())
        p.write(0, array, 0, array.size)
        return p
    }

    companion object {
        private val logger = LoggerFactory.getLogger(CredManagerBackedSecureStore::class.java)
        private val credAdvapi32Instance: CredAdvapi32
            get() = if (SystemHelper.isWindows()) {
                CredAdvapi32.INSTANCE
            } else {
                logger.warn(
                    "Returning a dummy library on non Windows platform.  " +
                            "This is a bug unless you are testing."
                )

                // Return a dummy on other platforms
                object : CredAdvapi32 {
                    @Throws(LastErrorException::class)
                    override fun CredRead(targetName: String?, type: Int, flags: Int, pcredential: CredAdvapi32.PCREDENTIAL?): Boolean {
                        return false
                    }

                    @Throws(LastErrorException::class)
                    override fun CredWrite(credential: CredAdvapi32.CREDENTIAL?, flags: Int): Boolean {
                        return false
                    }

                    @Throws(LastErrorException::class)
                    override fun CredDelete(targetName: String?, type: Int, flags: Int): Boolean {
                        return false
                    }

                    @Throws(LastErrorException::class)
                    override fun CredFree(credential: Pointer?) {
                    }
                }
            }
    }
}