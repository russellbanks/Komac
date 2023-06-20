package data.shared

import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.http.Url
import io.menu.prompts.UrlPrompt
import io.menu.prompts.UrlValidationRules
import kotlinx.datetime.LocalDate
import network.Http
import network.HttpUtils
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import okio.FileSystem
import schemas.manifest.InstallerManifest
import utils.FileAnalyser
import utils.Zip
import utils.extension
import utils.findArchitecture
import utils.hashSha256
import utils.msi.Msi
import utils.msix.Msix
import utils.msix.MsixBundle

object InstallerUrl : UrlPrompt {
    override val name: String = "Installer URL"

    override val description: String = "Download URL to the installer"

    override val validationRules: UrlValidationRules = UrlValidationRules(
        isRequired = true
    )

    suspend fun Terminal.downloadInstaller(
        packageIdentifier: String,
        packageVersion: String,
        installerUrl: Url,
        fileSystem: FileSystem = FileSystem.SYSTEM
    ): DownloadResult {
        lateinit var fileAnalyser: FileAnalyser
        lateinit var downloadedFile: HttpUtils.DownloadedFile
        var zip: Zip? = null
        val progress = getDownloadProgressBar(installerUrl).apply(ProgressAnimation::start)
        downloadedFile = Http.client.downloadFile(installerUrl, packageIdentifier, packageVersion, progress, fileSystem)
        progress.clear()
        fileAnalyser = FileAnalyser(downloadedFile.path)
        if (downloadedFile.path.extension.lowercase() == InstallerManifest.InstallerType.ZIP.toString()) {
            zip = Zip(zip = downloadedFile.path).also { it.prompt(this) }
        }
        return DownloadResult(
            releaseDate = downloadedFile.lastModified,
            scope = fileAnalyser.scope,
            installerSha256 = downloadedFile.path.hashSha256(),
            installerType = fileAnalyser.installerType,
            upgradeBehavior = fileAnalyser.upgradeBehaviour,
            architecture = installerUrl.findArchitecture() ?: fileAnalyser.architecture,
            publisherDisplayName = fileAnalyser.publisherDisplayName,
            msix = fileAnalyser.msix,
            msixBundle = fileAnalyser.msixBundle,
            msi = fileAnalyser.msi,
            zip = zip
        ).also {
            fileSystem.delete(downloadedFile.path)
            downloadedFile.removeFileDeletionHook()
        }
    }

    data class DownloadResult(
        val releaseDate: LocalDate?,
        val scope: InstallerManifest.Scope?,
        val installerSha256: String,
        val installerType: InstallerManifest.InstallerType?,
        val upgradeBehavior: InstallerManifest.UpgradeBehavior?,
        val architecture: InstallerManifest.Installer.Architecture,
        val publisherDisplayName: String?,
        val msix: Msix?,
        val msixBundle: MsixBundle?,
        val msi: Msi?,
        val zip: Zip?
    )

    fun Terminal.msixBundleDetection(msixBundle: MsixBundle?) {
        if (msixBundle != null) {
            println(
                verticalLayout {
                    cell(
                        (colors.brightGreen + colors.bold)(
                            "${msixBundle.packages?.size} packages have been detected inside the MSIX Bundle:"
                        )
                    )
                    msixBundle.packages?.forEachIndexed { index, individualPackage ->
                        cell(colors.brightGreen("Package ${index.inc()}/${msixBundle.packages?.size}"))
                        listOf(
                            "Architecture" to individualPackage.processorArchitecture,
                            "Version" to individualPackage.version,
                            "Minimum version" to individualPackage.minVersion,
                            "Platform" to individualPackage.targetDeviceFamily
                        ).forEach { (text, value) ->
                            if (value != null) {
                                var newText = text
                                var newValue = value
                                if (value is List<*>) {
                                    if (value.size > 1) newText = "${text}s"
                                    newValue = value.joinToString()
                                }
                                cell(colors.brightWhite("${" ".repeat(3)} $newText: $newValue"))
                            }
                        }
                    }
                }
            )
            println()
            info("All packages inside the MSIX Bundle will be added as separate installers in the manifest")
            println()
        }
    }
}
