package msix

import hashing.Hashing
import org.w3c.dom.Document
import schemas.InstallerManifest
import java.io.File
import java.util.zip.ZipFile
import javax.xml.parsers.DocumentBuilderFactory

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
                val xmlDocument: Document = zip.getInputStream(appxManifest).use {
                    DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(it)
                }
                val xmlPackages = xmlDocument.getElementsByTagName("Package")
                val packagesList = List(xmlPackages.length) { i -> xmlPackages.item(i) }
                val applicationsList = packagesList
                    .filter { it.attributes.getNamedItem("Type").nodeValue == "application" }
                packages = List(applicationsList.size) { applicationIndex ->
                    IndividualPackage(
                        version = applicationsList[applicationIndex].attributes.getNamedItem("Version").nodeValue,
                        targetDeviceFamily = List(applicationsList[applicationIndex].childNodes.length) { i ->
                            applicationsList[applicationIndex].childNodes.item(i)
                        }.filter { it.nodeName == "b4:Dependencies" }
                            .let { List(it[0].childNodes.length) { i -> it[0].childNodes.item(i) } }
                            .filter { it.nodeName == "b4:TargetDeviceFamily" }
                            .map { it.attributes.getNamedItem("Name").nodeValue }
                            .map { InstallerManifest.Platform.valueOf(it.replace(".", "")) },
                        minVersion = List(applicationsList[applicationIndex].childNodes.length) { i ->
                            applicationsList[applicationIndex].childNodes.item(i)
                        }.filter { it.nodeName == "b4:Dependencies" }
                            .let { List(it[0].childNodes.length) { i -> it[0].childNodes.item(i) } }
                            .find { it.nodeName == "b4:TargetDeviceFamily" }
                            ?.attributes?.getNamedItem("MinVersion")?.nodeValue,
                        processorArchitecture = InstallerManifest.Installer.Architecture.valueOf(
                            applicationsList[applicationIndex].attributes
                                .getNamedItem("Architecture").nodeValue.uppercase()
                        ),
                    )
                }
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
