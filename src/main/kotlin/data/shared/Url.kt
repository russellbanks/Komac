package data.shared

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.sun.jna.Platform
import data.ManifestData
import network.WebPageScraper
import utils.Zip
import utils.msi.Msi
import utils.msix.Msix
import utils.msix.MsixBundle
import github.GitHubDetection
import utils.extension
import utils.hashSha256
import io.ExitCode
import io.Prompts
import io.menu.yesNoMenu
import io.ktor.client.HttpClient
import io.ktor.client.network.sockets.ConnectTimeoutException
import io.ktor.client.request.head
import io.ktor.http.Url
import io.ktor.http.isSuccess
import java.net.ConnectException
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import okio.FileSystem
import schemas.manifest.InstallerManifest
import utils.FileAnalyser
import utils.findArchitecture
import utils.getRedirectedUrl
import utils.isRedirect

object Url {
    suspend fun Terminal.installerDownloadPrompt(parameterUrl: Url? = null) = with(ManifestData) {
        installerUrl = parameterUrl ?: promptForInstaller()
        downloadInstaller()
        msixBundleDetection()
    }

    private suspend fun Terminal.promptForInstaller(): Url {
        println(colors.brightGreen(installerUrlInfo))
        return prompt(installerUrlConst) { input ->
            runBlocking { isUrlValid(url = Url(input), canBeBlank = false) }
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(Url(input.trim()))
        }?.let {
            println()
            promptIfRedirectedUrl(it)
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    private suspend fun Terminal.promptIfRedirectedUrl(installerUrl: Url): Url {
        val redirectedUrl = installerUrl.getRedirectedUrl()
        val shouldUseRedirectedUrl = redirectedUrl != installerUrl &&
            !installerUrl.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true)
        return if (shouldUseRedirectedUrl) {
            println(colors.brightYellow(redirectFound))
            println(colors.cyan("Discovered URL: $redirectedUrl"))
            if (yesNoMenu(default = true).prompt()) {
                val error = isUrlValid(url = redirectedUrl, canBeBlank = false)
                if (error == null) {
                    success("URL changed to $redirectedUrl")
                    println()
                    redirectedUrl
                } else {
                    warning(error)
                    warning(detectedUrlValidationFailed)
                    installerUrl
                }
            } else {
                info("Original URL Retained - Proceeding with $installerUrl")
                installerUrl
            }
        } else {
            installerUrl
        }
    }

    private suspend fun Terminal.downloadInstaller(fileSystem: FileSystem = FileSystem.SYSTEM) = with(ManifestData) {
        if (installers.map(InstallerManifest.Installer::installerUrl).contains(installerUrl)) {
            installers += installers.first { it.installerUrl == installerUrl }
            skipAddInstaller = true
        } else {
            if (installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                gitHubDetection = GitHubDetection(installerUrl)
            } else {
                pageScraper = WebPageScraper(installerUrl)
            }
            val progress = getDownloadProgressBar(installerUrl).apply(ProgressAnimation::start)
            val downloadedFile = Http.client.downloadFile(installerUrl, packageIdentifier, packageVersion, progress, fileSystem)
            progress.clear()
            releaseDate = downloadedFile.lastModified
            val fileAnalyser = FileAnalyser(downloadedFile.path, fileSystem)
            installerType = fileAnalyser.installerType
            architecture = installerUrl.findArchitecture() ?: fileAnalyser.architecture
            scope = fileAnalyser.scope
            upgradeBehavior = fileAnalyser.upgradeBehaviour
            installerSha256 = gitHubDetection?.sha256 ?: downloadedFile.path.hashSha256(fileSystem)
            when (downloadedFile.path.extension.lowercase()) {
                InstallerManifest.InstallerType.MSIX.toString(),
                InstallerManifest.InstallerType.APPX.toString() -> msix = Msix(downloadedFile.path)
                MsixBundle.msixBundleConst,
                MsixBundle.appxBundleConst -> msixBundle = MsixBundle(downloadedFile.path)
                InstallerManifest.InstallerType.MSI.toString() -> if (Platform.isWindows()) {
                    msi = Msi(downloadedFile.path, fileSystem)
                }
                InstallerManifest.InstallerType.ZIP.toString() -> zip = Zip(
                    zip = downloadedFile.path,
                    terminal = this@downloadInstaller
                )
            }
            with(downloadedFile) {
                fileSystem.delete(path)
                removeFileDeletionHook()
            }
        }
    }

    private fun Terminal.msixBundleDetection() = with(ManifestData) {
        if (msixBundle != null) {
            println(
                verticalLayout {
                    cell(
                        (colors.brightGreen + colors.bold)(
                            "${msixBundle?.packages?.size} packages have been detected inside the MSIX Bundle:"
                        )
                    )
                    msixBundle?.packages?.forEachIndexed { index, individualPackage ->
                        cell(colors.brightGreen("Package ${index.inc()}/${msixBundle?.packages?.size}"))
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

    suspend fun isUrlValid(url: Url?, canBeBlank: Boolean, client: HttpClient = Http.client): String? {
        return when {
            url == null -> null
            url == Url("") && canBeBlank -> null
            url == Url("") -> Errors.blankInput(installerUrlConst)
            url.toString().length > maxLength -> Errors.invalidLength(max = maxLength)
            !url.toString().matches(regex) -> Errors.invalidRegex(regex)
            else -> client.checkUrlResponse(url)
        }
    }

    private suspend fun HttpClient.checkUrlResponse(url: Url): String? {
        return config { followRedirects = false }.use {
            try {
                val installerUrlResponse = it.head(url)
                if (!installerUrlResponse.status.isSuccess() && !installerUrlResponse.status.isRedirect) {
                    Errors.unsuccessfulUrlResponse(installerUrlResponse)
                } else {
                    null
                }
            } catch (_: ConnectTimeoutException) {
                Errors.connectionTimeout
            } catch (_: ConnectException) {
                Errors.connectionFailure
            }
        }
    }

    private const val installerUrlInfo = "${Prompts.required} Enter the download url to the installer"

    private const val redirectFound = "The URL is redirected. Would you like to use the destination URL instead?"

    private const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

    private const val installerUrlConst = "Installer Url"

    private const val maxLength = 2048
    private const val pattern = "^([Hh][Tt][Tt][Pp][Ss]?)://.+$"
    private val regex = Regex(pattern)
}
