package token

/**
 * A security token, usually acquired by some authentication and identity services.
 */
@JvmInline
value class TokenData(val value: String)
