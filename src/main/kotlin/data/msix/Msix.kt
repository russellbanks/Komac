package data.msix

import hashing.Hashing
import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import schemas.manifest.InstallerManifest
import java.io.File
import java.util.zip.ZipFile

class Msix(msixFile: File) {
    var displayName: String? = null
    private var displayVersion: String? = null
    var publisherDisplayName: String? = null
    var signatureSha256: String? = null
    var targetDeviceFamily: InstallerManifest.Platform? = null
    var minVersion: String? = null
    var description: String? = null
    var processorArchitecture: InstallerManifest.Installer.Architecture? = null
    private var packageFamilyName: String? = null

    init {
        val validExtensions = listOf(InstallerManifest.InstallerType.APPX, InstallerManifest.InstallerType.MSIX)
            .map { it.toString() }
        require(msixFile.extension.lowercase() in validExtensions) {
            "File extension must be one of the following: ${validExtensions.joinToString(", ")}"
        }
        ZipFile(msixFile).use { zip ->
            zip.getEntry(appxManifestXml)?.let { appxManifest ->
                val document = zip.getInputStream(appxManifest)
                    .use { htmlDocument(it) }
                    .let { Doc(document = it.document, relaxed = true) }
                val properties = document.findFirst("Properties")
                val targetDeviceFamilyAttribute = document.findFirst("TargetDeviceFamily")
                val identity = document.findFirst("Identity")
                targetDeviceFamily = targetDeviceFamilyAttribute
                    .attribute("Name".lowercase())
                    .ifBlank { null }
                    ?.replace(".", "")
                    ?.let { InstallerManifest.Platform.valueOf(it) }
                displayVersion = identity
                    .attribute("Version".lowercase())
                    .ifBlank { null }
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
            }
            zip.getEntry(appxSignatureP7x)?.let { appxSignature ->
                val digest = Hashing.Algorithms.SHA256
                zip.getInputStream(appxSignature).use { Hashing.updateDigest(it, digest) }
                signatureSha256 = Hashing.buildHash(digest.digest())
            }
        }
    }

    private fun getPackageFamilyName(identityName: String, identityPublisher: String): String {
        val hashPart = Hashing.Algorithms.SHA256
            .digest(identityPublisher.toByteArray(Charsets.UTF_16))
            .take(8)
            .flatMap { Integer.toBinaryString(it.toInt() and 0xff).padStart(8, '0').asIterable() }
            .toMutableList()
            .apply { add(0, '0') }
            .chunked(5)
            .map { "0123456789ABCDEFGHJKMNPQRSTVWXYZ"[it.joinToString("").toInt(2)] }
            .joinToString("")
            .lowercase()
        return "${identityName}_$hashPart"
    }

    fun resetExceptShared() {
        signatureSha256 = null
        targetDeviceFamily = null
        minVersion = null
        description = null
        processorArchitecture = null
        packageFamilyName = null
    }

    companion object {
        const val appxManifestXml = "AppxManifest.xml"
        const val appxSignatureP7x = "AppxSignature.p7x"
    }
}
