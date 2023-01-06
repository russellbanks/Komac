package msix

import hashing.Hashing
import org.w3c.dom.Document
import schemas.InstallerManifest
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipFile
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.xpath.XPath
import javax.xml.xpath.XPathConstants
import javax.xml.xpath.XPathFactory

data class Msix(
    val msixFile: File,
    var displayName: String? = null,
    var displayVersion: String? = null,
    var publisherDisplayName: String? = null,
    var signatureSha256: String? = null,
    var targetDeviceFamily: InstallerManifest.Platform? = null,
    var minVersion: String? = null,
    var description: String? = null,
    var processorArchitecture: InstallerManifest.Installer.Architecture? = null,
) {
    init {
        val validExtensions = listOf("appx", "appxbundle", "msix", "msixbundle")
        require(msixFile.extension.lowercase() in validExtensions) {
            "File extension must be one of the following: ${validExtensions.joinToString(", ")}"
        }
        ZipFile(msixFile).use { zip ->
            zip.getAppxManifestXml(msixFile)?.let { appxManifest ->
                val xPath: XPath = XPathFactory.newInstance().newXPath()
                val xmlDocument: Document = zip.getInputStream(appxManifest).use {
                    DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(it)
                }
                targetDeviceFamily = (
                    xPath.compile("/Package/Dependencies/TargetDeviceFamily/@Name")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                    ?.replace(".", "")
                    ?.let { InstallerManifest.Platform.valueOf(it) }
                displayVersion = (
                    xPath.compile("/Package/Identity/@Version")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                displayName = (
                    xPath.compile("/Package/Properties/DisplayName/text()")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                publisherDisplayName = (
                    xPath.compile("/Package/Properties/PublisherDisplayName/text()")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                minVersion = (
                    xPath.compile("/Package/Dependencies/TargetDeviceFamily/@MinVersion")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                description = (
                    xPath.compile("/Package/Properties/Description/text()")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    ).takeIf { it.isNotBlank() }
                processorArchitecture = (
                    xPath
                        .compile("/Package/Identity/@ProcessorArchitecture")
                        .evaluate(xmlDocument, XPathConstants.STRING) as String
                    )
                    .takeIf { it.isNotBlank() }
                    ?.let { InstallerManifest.Installer.Architecture.valueOf(it.uppercase()) }
            }
            zip.getEntry(appxSignatureP7x)?.let { appxSignature ->
                val digest = Hashing.Algorithms.SHA256
                zip.getInputStream(appxSignature).use { Hashing.updateDigest(it, digest) }
                signatureSha256 = Hashing.buildHash(digest.digest())
            }
        }
    }

    private fun ZipFile.getAppxManifestXml(file: File): ZipEntry? {
        return when (file.extension) {
            "msix" -> getEntry(appxManifestXml)
            "msixbundle" -> getEntry("$appxManifestFolder/$appxBundleManifestXml")
            else -> null
        }
    }

    companion object {
        const val appxManifestXml = "AppxManifest.xml"
        const val appxManifestFolder = "AppxMetadata"
        const val appxBundleManifestXml = "AppxBundleManifest.xml"
        const val appxSignatureP7x = "AppxSignature.p7x"
    }
}
