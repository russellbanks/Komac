package detection.files.msix

import hashing.Hashing

object MsixUtils {
    private const val firstEightBytes = 8
    private const val hex255 = 0xff
    private const val binaryRadix = 2
    private const val bitGroupsSize = 5
    private const val padLength = 8

    /**
     * Generates the package family name for a given identity name and identity publisher.
     *
     * The algorithm takes the following steps:
     * 1. Calculate the SHA-256 hash of the byte representation of the UTF-16 identity publisher.
     * 2. Take the first 8 bytes (64 bits) of the SHA-256 hash.
     * 3. Concatenate each byte of the first 8 bytes, and convert them to binary representation.
     * 4. Pad the binary value by a single zero bit to the right (left shift all bits).
     * 5. Group the bits in groups of 5.
     * 6. For each group, convert the bit representation to an index of the string "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
     * 7. Join the letters together and make them lowercase.
     * 8. Append the hash part to the identity name with an underscore as a separator.
     *
     * @param identityName a string representing the identity name.
     * @param identityPublisher a UTF-16 string representing the identity publisher.
     * @return the package family name generated using the algorithm.
     */
    fun getPackageFamilyName(identityName: String, identityPublisher: String): String {
        val hashPart = Hashing.Algorithms.SHA256
            .digest(identityPublisher.toByteArray(Charsets.UTF_16LE))
            .take(firstEightBytes)
            .map { it.toInt() and hex255 }
            .joinToString("") { it.toString(binaryRadix).padStart(padLength, '0') }
            .plus("0")
            .chunked(bitGroupsSize)
            .map { "0123456789ABCDEFGHJKMNPQRSTVWXYZ"[it.toInt(binaryRadix)] }
            .joinToString("")
            .lowercase()
        return "${identityName}_$hashPart"
    }
}
