package data.msix

import hashing.Hashing
import it.skrape.core.htmlDocument
import it.skrape.selects.Doc
import it.skrape.selects.attribute
import schemas.manifest.InstallerManifest
import java.io.File
import java.util.zip.ZipFile

class MsixBundle(msixBundleFile: File) {
    var signatureSha256: String? = null
    var packages: List<IndividualPackage>? = null

    init {
        require(msixBundleFile.extension.lowercase() in listOf(appxBundleConst, msixBundleConst)) {
            "File must be an ${InstallerManifest.InstallerType.MSIX}"
        }
        require(msixBundleFile.extension.lowercase() == msixBundleConst) { "File must be an $msixBundleConst" }
        ZipFile(msixBundleFile).use { zip ->
            zip.getEntry("$appxManifestFolder/$appxBundleManifestXml")?.let { appxManifest ->
                packages = zip.getInputStream(appxManifest)
                    .use { htmlDocument(it) }
                    .let { Doc(document = it.document, relaxed = true) }
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
                                        ?.let { InstallerManifest.Platform.valueOf(it) }
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
                val digest = Hashing.Algorithms.SHA256
                zip.getInputStream(appxSignature).use { Hashing.updateDigest(it, digest) }
                signatureSha256 = Hashing.buildHash(digest.digest())
            }
        }
    }

    data class IndividualPackage(
        var version: String? = null,
        var targetDeviceFamily: List<InstallerManifest.Platform>? = null,
        var minVersion: String? = null,
        var processorArchitecture: InstallerManifest.Installer.Architecture? = null,
    )

    fun resetExceptShared() {
        signatureSha256 = null
        packages?.forEach {
            it.processorArchitecture = null
            it.minVersion = null
            it.targetDeviceFamily = null
        }
    }

    companion object {
        const val appxManifestFolder = "AppxMetadata"
        const val appxBundleManifestXml = "AppxBundleManifest.xml"
        const val appxSignatureP7x = "AppxSignature.p7x"
        const val msixBundleConst = "msixbundle"
        const val appxBundleConst = "appxbundle"
    }
}
