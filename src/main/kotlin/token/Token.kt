// Copyright (c) Microsoft. All rights reserved.
// Licensed under the MIT license. See License.txt in the project root.
package token

import com.microsoft.alm.helpers.Debug
import com.microsoft.alm.helpers.Guid
import com.microsoft.alm.helpers.NotImplementedException
import com.microsoft.alm.helpers.StringHelper
import com.microsoft.alm.secret.Secret
import com.microsoft.alm.secret.TokenType
import com.microsoft.alm.helpers.XmlHelper
import org.slf4j.LoggerFactory
import org.w3c.dom.Document
import org.w3c.dom.Element
import org.w3c.dom.Node
import java.nio.ByteBuffer
import java.util.Base64
import java.util.EnumSet
import java.util.UUID
import java.util.concurrent.atomic.AtomicReference

/**
 * A security token, usually acquired by some authentication and identity services.
 */
class Token : Secret {
    constructor(value: String, type: TokenType) {
        Debug.Assert(!StringHelper.isNullOrWhiteSpace(value), "The value parameter is null or invalid")
        // PORT NOTE: Java doesn't have the concept of out-of-range enums
        Type = type
        Value = value
    }

    constructor(value: String, typeName: String) {
        Debug.Assert(!StringHelper.isNullOrWhiteSpace(value), "The value parameter is null or invalid")
        Debug.Assert(!StringHelper.isNullOrWhiteSpace(typeName), "The typeName parameter is null or invalid")
        val type: AtomicReference<TokenType> = AtomicReference<TokenType>()
        require(getTypeFromFriendlyName(typeName, type)) { "Unexpected token type '$typeName' encountered" }
        Type = type.get()
        Value = value
    }
    // PORT NOTE: ADAL-specific constructor omitted
    /**
     * The type of the security token.
     */
    val Type: TokenType

    /**
     * The raw contents of the token.
     */
    val Value: String
    var targetIdentity: UUID = Guid.Empty
    fun toXml(document: Document): Element {
        val valueNode = document.createElement("value")
        val typeNode = document.createElement("Type")
        val typeValue = document.createTextNode(Type.toString())
        typeNode.appendChild(typeValue)
        valueNode.appendChild(typeNode)
        val tokenValueNode = document.createElement("Value")
        val valueValue = document.createTextNode(Value)
        tokenValueNode.appendChild(valueValue)
        valueNode.appendChild(tokenValueNode)
        if (Guid.Empty != targetIdentity) {
            val targetIdentityNode = document.createElement("targetIdentity")
            val targetIdentityValue = document.createTextNode(targetIdentity.toString())
            targetIdentityNode.appendChild(targetIdentityValue)
            valueNode.appendChild(targetIdentityNode)
        }
        return valueNode
    }

    /**
     * Compares an object to this [Token] for equality.
     *
     * @param obj The object to compare.
     * @return True is equal; false otherwise.
     */
    override fun equals(obj: Any?): Boolean {
        return operatorEquals(this, obj as? Token)
    }
    // PORT NOTE: Java doesn't support a specific overload (as per IEquatable<T>)
    /**
     * Gets a hash code based on the contents of the token.
     *
     * @return 32-bit hash code.
     */
    override fun hashCode(): Int {
        // PORT NOTE: Java doesn't have unchecked blocks; the default behaviour is apparently equivalent.
        run { return Type.value * Value.hashCode() }
    }

    /**
     * Converts the token to a human friendly string.
     *
     * @return Humanish name of the token.
     */
    override fun toString(): String {
        val value = AtomicReference<String>()
        return if (getFriendlyNameFromType(Type, value)) value.get() else super.toString()
    }

    fun contributeHeader(headers: MutableMap<String?, String?>) {
        // different types of tokens are packed differently
        when (Type) {
            TokenType.Access -> {
                val prefix = "Bearer"
                headers["Authorization"] = "$prefix $Value"
            }

            TokenType.Personal -> {
                val authData = StringHelper.UTF8GetBytes("PersonalAccessToken:$Value")
                val base64EncodedAuthData: String = Base64.getEncoder().encodeToString(authData)
                headers["Authorization"] = "Basic $base64EncodedAuthData"
            }

            TokenType.Federated -> throw NotImplementedException(449222)
            else -> {
                val template = "Tokens of type '%1\$s' cannot be used for headers."
                val message = String.format(template, Type)
                throw IllegalStateException(message)
            }
        }
    }

    companion object {
        private val logger = LoggerFactory.getLogger(Token::class.java)
        private const val sizeofTokenType = 4
        private const val sizeofGuid = 16
        fun getFriendlyNameFromType(type: TokenType, name: AtomicReference<String>): Boolean {
            // PORT NOTE: Java doesn't have the concept of out-of-range enums
            name.set(null)
            name.set(if (type.description == null) type.toString() else type.description)
            return name.get() != null
        }

        fun getTypeFromFriendlyName(name: String, type: AtomicReference<TokenType>): Boolean {
            Debug.Assert(!StringHelper.isNullOrWhiteSpace(name), "The name parameter is null or invalid")
            type.set(TokenType.Unknown)
            for (value in EnumSet.allOf(TokenType::class.java)) {
                type.set(value)
                val typename = AtomicReference<String>()
                if (getFriendlyNameFromType(type.get(), typename)) {
                    if (name.equals(typename.get(), ignoreCase = true)) return true
                }
            }
            return false
        }

        fun fromXml(tokenNode: Node): Token {
            val value: Token
            var tokenValue: String? = null
            var tokenType: TokenType? = null
            var targetIdentity = Guid.Empty
            val propertyNodes = tokenNode.childNodes
            for (v in 0 until propertyNodes.length) {
                val propertyNode = propertyNodes.item(v)
                when (propertyNode.nodeName) {
                    "Type" -> {
                        tokenType = TokenType.valueOf(XmlHelper.getText(propertyNode))
                    }
                    "Value" -> {
                        tokenValue = XmlHelper.getText(propertyNode)
                    }
                    "targetIdentity" -> {
                        targetIdentity = UUID.fromString(XmlHelper.getText(propertyNode))
                    }
                }
            }
            value = Token(tokenValue!!, tokenType!!)
            value.targetIdentity = targetIdentity
            return value
        }

        fun deserialize(bytes: ByteArray?, type: TokenType?, tokenReference: AtomicReference<Token?>): Boolean {
            Debug.Assert(bytes != null, "The bytes parameter is null")
            Debug.Assert(bytes!!.isNotEmpty(), "The bytes parameter is too short")
            Debug.Assert(type != null, "The type parameter is invalid")
            tokenReference.set(null)
            try {
                val preamble = sizeofTokenType + sizeofGuid
                if (bytes.size > preamble) {
                    var readType: TokenType
                    var targetIdentity: UUID
                    val p = ByteBuffer.wrap(bytes) // PORT NOTE: ByteBuffer is closest to "fixed"
                    run {
                        readType = TokenType.fromValue(Integer.reverseBytes(p.int))
                        val guidBytes = ByteArray(16)
                        p[guidBytes]
                        targetIdentity = Guid.fromBytes(guidBytes)
                    }
                    if (readType === type) {
                        val value = StringHelper.UTF8GetString(bytes, preamble, bytes.size - preamble)
                        if (!StringHelper.isNullOrWhiteSpace(value)) {
                            tokenReference.set(Token(value, type))
                            tokenReference.get()!!.targetIdentity = targetIdentity
                        }
                    }
                }

                // if value hasn't been set yet, fall back to old format decode
                if (tokenReference.get() == null) {
                    val value = StringHelper.UTF8GetString(bytes)
                    if (!StringHelper.isNullOrWhiteSpace(value)) {
                        tokenReference.set(Token(value, type!!))
                    }
                }
            } catch (throwable: Throwable) {
                logger.debug("   token deserialization error")
            }
            return tokenReference.get() != null
        }

        fun serialize(token: Token?, byteReference: AtomicReference<ByteArray?>): Boolean {
            Debug.Assert(token != null, "The token parameter is null")
            Debug.Assert(!StringHelper.isNullOrWhiteSpace(token!!.Value), "The token.Value is invalid")
            byteReference.set(null)
            try {
                val utf8bytes = StringHelper.UTF8GetBytes(token.Value)
                val bytes =
                    ByteBuffer.allocate(utf8bytes.size + sizeofTokenType + sizeofGuid)

                // PORT NOTE: "fixed" block pointer arithmetic and casting avoided
                run {
                    bytes.putInt(Integer.reverseBytes(token.Type.value))
                    bytes.put(Guid.toBytes(token.targetIdentity))
                }
                bytes.put(utf8bytes)
                byteReference.set(bytes.array())
            } catch (t: Throwable) {
                logger.debug("   token serialization error")
            }
            return byteReference.get() != null
        }

        /**
         * Compares two tokens for equality.
         *
         * @param token1 Token to compare.
         * @param token2 Token to compare.
         * @return True if equal; false otherwise.
         */
        fun operatorEquals(token1: Token?, token2: Token?): Boolean {
            if (token1 === token2) return true
            return if (token1 == null || null == token2) false else (token1.Type === token2.Type
                    && token1.Value.equals(token2.Value, ignoreCase = true))
        }

        /**
         * Compares two tokens for inequality.
         *
         * @param token1 Token to compare.
         * @param token2 Token to compare.
         * @return False if equal; true otherwise.
         */
        fun operatorNotEquals(token1: Token?, token2: Token?): Boolean {
            return !operatorEquals(token1, token2)
        }
    }
}