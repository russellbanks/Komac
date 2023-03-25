package detection.files.msix

import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import it.skrape.selects.attribute
import okio.HashingSink.Companion.sha256
import okio.blackholeSink
import okio.buffer
import okio.source
import schemas.manifest.InstallerManifest
import utils.MsixUtils
import java.io.File
import java.util.zip.ZipFile

class MsixBundle(msixBundleFile: File) {
    var signatureSha256: String? = null
    var packageFamilyName: String? = null
    var packages: List<IndividualPackage>? = null

    init {
        require(msixBundleFile.extension.lowercase() in listOf(appxBundleConst, msixBundleConst)) {
            "File must be an ${InstallerManifest.InstallerType.MSIX}"
        }
        require(msixBundleFile.extension.lowercase() == msixBundleConst) { "File must be an $msixBundleConst" }
        ZipFile(msixBundleFile).use { zip ->
            zip.getEntry("$appxManifestFolder/$appxBundleManifestXml")?.let { appxManifest ->
                packages = Doc(document = zip.getInputStream(appxManifest).use(::htmlDocument).document, relaxed = true)
                    .also {
                        val identity = it.findFirst("Identity")
                        packageFamilyName = MsixUtils.getPackageFamilyName(
                            identityName = identity.attribute("Name".lowercase()),
                            identityPublisher = identity.attribute("Publisher".lowercase())
                        )
                    }
                    .findAll("Package")
                    .filter { it.attribute("Type".lowercase()).equals(other = "Application", ignoreCase = true) }
                    .map { packageElement ->
                        val targetDeviceFamily = packageElement.findAll("*|TargetDeviceFamily".lowercase())
                        IndividualPackage(
                            version = packageElement.attribute("Version".lowercase()).ifBlank { null },
                            targetDeviceFamily = targetDeviceFamily
                                .mapNotNull { targetDeviceFamilyElement ->
                                    targetDeviceFamilyElement.attribute("Name".lowercase())
                                        .ifBlank { null }
                                        ?.replace(".", "")
                                        ?.let(InstallerManifest.Platform::valueOf)
                                }
                                .ifEmpty { null },
                            minVersion = targetDeviceFamily.attribute("MinVersion".lowercase()).ifBlank { null },
                            processorArchitecture = packageElement
                                .attribute("Architecture".lowercase())
                                .ifBlank { null }
                                ?.let { InstallerManifest.Installer.Architecture.valueOf(it.uppercase()) }
                        )
                    }
                    .ifEmpty { null }
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

    data class IndividualPackage(
        val version: String? = null,
        val targetDeviceFamily: List<InstallerManifest.Platform>? = null,
        val minVersion: String? = null,
        val processorArchitecture: InstallerManifest.Installer.Architecture? = null,
    )

    companion object {
        const val appxManifestFolder = "AppxMetadata"
        const val appxBundleManifestXml = "AppxBundleManifest.xml"
        const val appxSignatureP7x = "AppxSignature.p7x"
        const val msixBundleConst = "msixbundle"
        const val appxBundleConst = "appxbundle"
    }
}
