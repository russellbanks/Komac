package utils.msix

import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import it.skrape.selects.attribute
import okio.BufferedSource
import okio.FileSystem
import okio.Path
import okio.Path.Companion.toPath
import okio.buffer
import okio.openZip
import schemas.manifest.InstallerManifest
import utils.extension
import utils.hashSha256

class MsixBundle(msixBundleFile: Path, fileSystem: FileSystem = FileSystem.SYSTEM) {
    var signatureSha256: String? = null
    var packageFamilyName: String? = null
    var packages: List<IndividualPackage>? = null

    init {
        require(msixBundleFile.extension.lowercase() in listOf(appxBundleConst, msixBundleConst)) {
            "File must be an $msixBundleConst or $appxBundleConst"
        }
        val msixBundleFileSystem = fileSystem.openZip(msixBundleFile)
        val appxManifestXml = msixBundleFileSystem
            .source(appxManifestFolder.toPath() / appxBundleManifestXml)
            .buffer()
            .use(BufferedSource::readUtf8)
        packages = Doc(htmlDocument(appxManifestXml).document, relaxed = true)
            .also {
                val identity = it.findFirst("Identity")
                packageFamilyName = Msix.getPackageFamilyName(
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
        signatureSha256 = appxSignatureP7x.toPath().hashSha256(msixBundleFileSystem)
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
