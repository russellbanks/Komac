package detection.files.msix

import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import java.io.File
import java.util.zip.ZipFile
import okio.HashingSink.Companion.sha256
import okio.blackholeSink
import okio.buffer
import okio.source
import schemas.manifest.InstallerManifest
import utils.MsixUtils

class Msix(msixFile: File) {
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
            .map(InstallerManifest.InstallerType::toString)
        require(msixFile.extension.lowercase() in validExtensions) {
            "File extension must be one of the following: ${validExtensions.joinToString()}"
        }
        ZipFile(msixFile).use { zip ->
            zip.getEntry(appxManifestXml)?.let { appxManifest ->
                val document = Doc(zip.getInputStream(appxManifest).use(::htmlDocument).document, relaxed = true)
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
                packageFamilyName = MsixUtils.getPackageFamilyName(
                    identityName = identity.attribute("Name".lowercase()),
                    identityPublisher = identity.attribute("Publisher".lowercase())
                )
            }
            zip.getEntry(appxSignatureP7x)?.let { appxSignature ->
                zip.getInputStream(appxSignature).use {
                    sha256(blackholeSink()).use { hashingSink ->
                        it.source().buffer().use { source ->
                            source.readAll(hashingSink)
                            signatureSha256 = hashingSink.hash.hex()
                        }
                    }
                }
            }
        }
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
