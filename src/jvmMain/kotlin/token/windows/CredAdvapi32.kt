package token.windows

import com.sun.jna.LastErrorException
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.Structure.FieldOrder
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.win32.StdCallLibrary
import com.sun.jna.win32.W32APIOptions

/**
 * This class exposes functions from credential manager on Windows platform
 * via JNA.
 *
 * Please refer to MSDN documentations for each method usage pattern
 */
@Suppress("FunctionName", "VariableNaming", "Unused")
internal interface CredAdvapi32 : StdCallLibrary {

    /**
     * The CREDENTIAL structure contains an individual credential
     *
     * https://msdn.microsoft.com/en-us/library/windows/desktop/aa374788(v=vs.85).aspx
     *
     * typedef struct _CREDENTIAL {
     * DWORD                 Flags;
     * DWORD                 Type;
     * LPTSTR                TargetName;
     * LPTSTR                Comment;
     * FILETIME              LastWritten;
     * DWORD                 CredentialBlobSize;
     * LPBYTE                CredentialBlob;
     * DWORD                 Persist;
     * DWORD                 AttributeCount;
     * PCREDENTIAL_ATTRIBUTE Attributes;
     * LPTSTR                TargetAlias;
     * LPTSTR                UserName;
     * } CREDENTIAL, *PCREDENTIAL;
     */
    @FieldOrder(
        "Flags", "Type", "TargetName", "Comment", "LastWritten", "CredentialBlobSize", "CredentialBlob", "Persist",
        "AttributeCount", "Attributes", "TargetAlias", "UserName"
    )
    class CREDENTIAL : Structure {
        constructor() : super()
        constructor(memory: Pointer) : super(memory) {
            read()
        }

        /**
         * A bit member that identifies characteristics of the credential. Undefined bits should be initialized
         * as zero and not otherwise altered to permit future enhancement.
         *
         * See MSDN doc for all possible flags
         */
        @JvmField var Flags: Int = 0

        /**
         * The type of the credential. This member cannot be changed after the credential is created.
         *
         * See MSDN doc for all possible types
         */
        @JvmField var Type = 0

        /**
         * The name of the credential. The TargetName and Type members uniquely identify the credential.
         * This member cannot be changed after the credential is created. Instead, the credential with the old
         * name should be deleted and the credential with the new name created.
         *
         * See MSDN doc for additional requirement
         */
        @JvmField var TargetName: String? = null

        /**
         * A string comment from the user that describes this credential. This member cannot be longer than
         * CRED_MAX_STRING_LENGTH (256) characters.
         */
        @JvmField var Comment: String? = null

        /**
         * The time, in Coordinated Universal Time (Greenwich Mean Time), of the last modification of the credential.
         * For write operations, the value of this member is ignored.
         */
        @JvmField var LastWritten: FILETIME? = null

        /**
         * The size, in bytes, of the CredentialBlob member. This member cannot be larger than
         * CRED_MAX_CREDENTIAL_BLOB_SIZE (512) bytes.
         */
        @JvmField var CredentialBlobSize = 0

        /**
         * Secret data for the credential. The CredentialBlob member can be both read and written.
         * If the Type member is CRED_TYPE_DOMAIN_PASSWORD, this member contains the plaintext Unicode password
         * for UserName. The CredentialBlob and CredentialBlobSize members do not include a trailing zero character.
         * Also, for CRED_TYPE_DOMAIN_PASSWORD, this member can only be read by the authentication packages.
         *
         * If the Type member is CRED_TYPE_DOMAIN_CERTIFICATE, this member contains the clear test
         * Unicode PIN for UserName. The CredentialBlob and CredentialBlobSize members do not include a trailing
         * zero character. Also, this member can only be read by the authentication packages.
         *
         * If the Type member is CRED_TYPE_GENERIC, this member is defined by the application.
         * Credentials are expected to be portable. Applications should ensure that the data in CredentialBlob is
         * portable. The application defines the byte-endian and alignment of the data in CredentialBlob.
         */
        @JvmField var CredentialBlob: Pointer? = null

        /**
         * Defines the persistence of this credential. This member can be read and written.
         *
         * See MSDN doc for all possible values
         */
        @JvmField var Persist = 0

        /**
         * The number of application-defined attributes to be associated with the credential. This member can be
         * read and written. Its value cannot be greater than CRED_MAX_ATTRIBUTES (64).
         */
        @JvmField var AttributeCount = 0

        /**
         * Application-defined attributes that are associated with the credential. This member can be read
         * and written.
         */
        @JvmField var Attributes: Pointer? = null

        /**
         * Alias for the TargetName member. This member can be read and written. It cannot be longer than
         * CRED_MAX_STRING_LENGTH (256) characters.
         *
         * If the credential Type is CRED_TYPE_GENERIC, this member can be non-NULL, but the credential manager
         * ignores the member.
         */
        @JvmField var TargetAlias: String? = null

        /**
         * The username of the account used to connect to TargetName.
         * If the credential Type is CRED_TYPE_DOMAIN_PASSWORD, this member can be either a DomainName\UserName
         * or a UPN.
         *
         * If the credential Type is CRED_TYPE_DOMAIN_CERTIFICATE, this member must be a marshaled certificate
         * reference created by calling CredMarshalCredential with a CertCredential.
         *
         * If the credential Type is CRED_TYPE_GENERIC, this member can be non-NULL, but the credential manager
         * ignores the member.
         *
         * This member cannot be longer than CRED_MAX_USERNAME_LENGTH (513) characters.
         */
        @JvmField var UserName: String? = null
    }

    /**
     * Pointer to {@see CREDENTIAL} struct
     */
    @FieldOrder("credential")
    class PCREDENTIAL : Structure() {
        @JvmField var credential: Pointer? = null
    }

    /**
     * The CredRead function reads a credential from the user's credential set.
     *
     * The credential set used is the one associated with the logon session of the current token.
     * The token must not have the user's SID disabled.
     *
     * https://msdn.microsoft.com/en-us/library/windows/desktop/aa374804(v=vs.85).aspx
     *
     * @param targetName
     * String that contains the name of the credential to read.
     * @param type
     * Type of the credential to read. Type must be one of the CRED_TYPE_* defined types.
     * @param flags
     * Currently reserved and must be zero.
     * @param pcredential
     * Out - Pointer to a single allocated block buffer to return the credential.
     * Any pointers contained within the buffer are pointers to locations within this single allocated block.
     * The single returned buffer must be freed by calling `CredFree`.
     *
     * @return
     * True if CredRead succeeded, false otherwise
     *
     * @throws LastErrorException
     * GetLastError
     */
    @Throws(LastErrorException::class)
    fun CredRead(targetName: String?, type: Int, flags: Int, pcredential: PCREDENTIAL?): Boolean

    /**
     * The CredWrite function creates a new credential or modifies an existing credential in the user's credential set.
     * The new credential is associated with the logon session of the current token. The token must not have the
     * user's security identifier (SID) disabled.
     *
     * https://msdn.microsoft.com/en-us/library/windows/desktop/aa375187(v=vs.85).aspx
     *
     * @param credential
     * A CREDENTIAL structure to be written.
     * @param flags
     * Flags that control the function's operation. The following flag is defined.
     * CRED_PRESERVE_CREDENTIAL_BLOB:
     * The credential BLOB from an existing credential is preserved with the same
     * credential name and credential type. The CredentialBlobSize of the passed
     * in Credential structure must be zero.
     *
     * @return
     * True if CredWrite succeeded, false otherwise
     *
     * @throws LastErrorException
     * GetLastError
     */
    @Throws(LastErrorException::class)
    fun CredWrite(credential: CREDENTIAL?, flags: Int): Boolean

    /**
     * The CredDelete function deletes a credential from the user's credential set. The credential set used is the one
     * associated with the logon session of the current token. The token must not have the user's SID disabled.
     *
     * https://msdn.microsoft.com/en-us/library/windows/desktop/aa374787(v=vs.85).aspx
     *
     * @param targetName
     * String that contains the name of the credential to read.
     * @param type
     * Type of the credential to delete. Must be one of the CRED_TYPE_* defined types. For a list of the
     * defined types, see the Type member of the CREDENTIAL structure.
     * If the value of this parameter is CRED_TYPE_DOMAIN_EXTENDED, this function can delete a credential that
     * specifies a username when there are multiple credentials for the same target. The value of the TargetName
     * parameter must specify the username as Target|UserName.
     * @param flags
     * Reserved and must be zero.
     *
     * @return
     * True if CredDelete succeeded, false otherwise
     *
     * @throws LastErrorException
     * GetLastError
     */
    @Throws(LastErrorException::class)
    fun CredDelete(targetName: String?, type: Int, flags: Int): Boolean

    /**
     * The CredFree function frees a buffer returned by any of the credentials management functions.
     *
     * https://msdn.microsoft.com/en-us/library/windows/desktop/aa374796(v=vs.85).aspx
     *
     * @param credential
     * Pointer to CREDENTIAL to be freed
     *
     * @throws LastErrorException
     * GetLastError
     */
    @Throws(LastErrorException::class)
    fun CredFree(credential: Pointer?)

    companion object {
        val INSTANCE = Native.load("Advapi32", CredAdvapi32::class.java, W32APIOptions.UNICODE_OPTIONS) as CredAdvapi32

        /**
         * Type of Credential
         */
        const val CRED_TYPE_GENERIC = 1

        /**
         * Values of the Credential Persist field
         */
        const val CRED_PERSIST_LOCAL_MACHINE = 2
    }
}
