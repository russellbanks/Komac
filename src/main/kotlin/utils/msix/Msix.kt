package utils.msix

import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import okio.BufferedSource
import okio.ByteString.Companion.encode
import okio.FileSystem
import okio.Path
import okio.Path.Companion.toPath
import okio.buffer
import okio.openZip
import schemas.manifest.InstallerManifest
import utils.extension
import utils.hashSha256

class Msix(msixFile: Path, fileSystem: FileSystem = FileSystem.SYSTEM) {
    var displayName: String? = null
    private var displayVersion: String? = null
    var publisherDisplayName: String? = null
    var signatureSha256: String? = null
    var targetDeviceFamily: InstallerManifest.Platform? = null
    var minVersion: String? = null
    var description: String? = null
    var processorArchitecture: InstallerManifest.Installer.Architecture? = null
    var packageFamilyName: String? = null

    init {
        val validExtensions = listOf(InstallerManifest.InstallerType.APPX, InstallerManifest.InstallerType.MSIX)
            .map { it.name.lowercase() }
        require(msixFile.extension.lowercase() in validExtensions) {
            "File extension must be one of the following: ${validExtensions.joinToString()}"
        }
        val msixFileSystem = fileSystem.openZip(msixFile)
        val appxManifestXML = msixFileSystem.source(APPX_MANIFEST_XML.toPath()).buffer().use(BufferedSource::readUtf8)
        val document = Doc(htmlDocument(appxManifestXML).document, relaxed = true)
        val properties = document.findFirst("Properties")
        val targetDeviceFamilyAttribute = document.findFirst("TargetDeviceFamily")
        val identity = document.findFirst("Identity")
        targetDeviceFamily = targetDeviceFamilyAttribute
            .attribute("Name".lowercase())
            .ifBlank { null }
            ?.replace(".", "")
            ?.let(InstallerManifest.Platform::valueOf)
        displayVersion = identity.attribute("Version".lowercase()).ifBlank { null }
        displayName = properties.findFirst("DisplayName").text.ifBlank { null }
        publisherDisplayName = properties.findFirst("PublisherDisplayName").text.ifBlank { null }
        minVersion = targetDeviceFamilyAttribute.attribute("MinVersion".lowercase()).ifBlank { null }
        description = properties.findFirst("Description").text.ifBlank { null }
        processorArchitecture = identity
            .attribute("ProcessorArchitecture".lowercase())
            .ifBlank { null }
            ?.let { InstallerManifest.Installer.Architecture.valueOf(it.uppercase()) }
        packageFamilyName = getPackageFamilyName(
            identityName = identity.attribute("Name".lowercase()),
            identityPublisher = identity.attribute("Publisher".lowercase())
        )
        signatureSha256 = APPX_SIGNATURE_P7X.toPath().hashSha256(msixFileSystem)
    }

    companion object {
        const val APPX_MANIFEST_XML = "AppxManifest.xml"
        const val APPX_SIGNATURE_P7X = "AppxSignature.p7x"
        private const val HEX_255 = 0xFF
        private const val BASE_2 = 2
        private const val BIT_GROUPS_SIZE = 5
        private const val PAD_LENGTH = 8

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
            val hashPart = identityPublisher.encode(Charsets.UTF_16LE)
                .sha256()
                .substring(0, 8)
                .toByteArray()
                .map { it.toInt() and HEX_255 }
                .joinToString("") { it.toString(BASE_2).padStart(PAD_LENGTH, '0') }
                .plus('0')
                .chunked(BIT_GROUPS_SIZE)
                .map { "0123456789ABCDEFGHJKMNPQRSTVWXYZ"[it.toInt(BASE_2)] }
                .joinToString("")
                .lowercase()
            return "${identityName}_$hashPart"
        }
    }
}
