package token

/**
 * Secret store to hold the credentials.
 *
 * @param <E> a secret
</E> */
interface SecretStore<E : Token?> {
    /**
     * Retrieve a secret identified by the key from this store.
     *
     * If there is no secret identified by this key, return `null`
     *
     * @param key
     * for which a secret is associated with
     *
     * @return secret stored by this key, or `null`
     */
    operator fun get(key: String): E?

    /**
     * Remove the secret identified by the key from this store
     *
     * @param key
     * for which a secret is associated with
     *
     * @return `true` if secret is deleted successfully
     * `false` otherwise
     */
    fun delete(key: String): Boolean

    /**
     * Save the secret identified by the key to this store.  Replace existing secret if it exists.
     * @param key
     * for which a secret is associated with
     * @param secret
     * secret to be stored
     *
     * @return `true` if secret is added successfully
     * `false` otherwise
     */
    fun add(key: String, secret: E): Boolean
}
