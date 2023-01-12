package msix

import hashing.Hashing
import org.w3c.dom.Document
import schemas.InstallerManifest
import java.io.File
import java.util.zip.ZipFile
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.xpath.XPath
import javax.xml.xpath.XPathConstants
import javax.xml.xpath.XPathFactory

class Msix(msixFile: File) {
    var displayName: String? = null
    var displayVersion: String? = null
    var publisherDisplayName: String? = null
    var signatureSha256: String? = null
    var targetDeviceFamily: InstallerManifest.Platform? = null
    var minVersion: String? = null
    var description: String? = null
    var processorArchitecture: InstallerManifest.Installer.Architecture? = null
    var packageFamilyName: String? = null

    init {
        val validExtensions = listOf(InstallerManifest.InstallerType.APPX, InstallerManifest.InstallerType.MSIX)
            .map { it.toString() }
        require(msixFile.extension.lowercase() in validExtensions) {
            "File extension must be one of the following: ${validExtensions.joinToString(", ")}"
        }
        ZipFile(msixFile).use { zip ->
            zip.getEntry(appxManifestXml)?.let { appxManifest ->
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
                getPackageFamilyName(xPath, xmlDocument)
            }
            zip.getEntry(appxSignatureP7x)?.let { appxSignature ->
                val digest = Hashing.Algorithms.SHA256
                zip.getInputStream(appxSignature).use { Hashing.updateDigest(it, digest) }
                signatureSha256 = Hashing.buildHash(digest.digest())
            }
        }
    }

    private fun getPackageFamilyName(xPath: XPath, xmlDocument: Document) {
        val identityName =
            xPath.compile("/Package/Identity/@Name").evaluate(xmlDocument, XPathConstants.STRING) as String
        val identityPublisher =
            xPath.compile("/Package/Identity/@Publisher").evaluate(xmlDocument, XPathConstants.STRING) as String
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
        packageFamilyName = "${identityName}_$hashPart"
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
