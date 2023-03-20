package data.shared

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.sun.jna.Platform
import data.AllManifestData
import data.GitHubImpl
import detection.PageScraper
import detection.files.Zip
import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import detection.github.GitHubDetection
import input.ExitCode
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.network.sockets.ConnectTimeoutException
import io.ktor.client.request.head
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import io.ktor.http.isSuccess
import kotlinx.coroutines.runBlocking
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import schemas.manifest.InstallerManifest
import utils.FileAnalyser
import utils.Hashing.hash
import utils.findArchitecture
import utils.getRedirectedUrl
import utils.isRedirect
import utils.yesNoMenu
import java.net.ConnectException

object Url {
    suspend fun Terminal.installerDownloadPrompt(
        allManifestData: AllManifestData,
        client: HttpClient,
        gitHubImpl: GitHubImpl,
        parameterUrl: Url? = null
    ) = with(allManifestData) {
        installerUrl = parameterUrl ?: promptForInstaller(client)
        downloadInstaller(allManifestData, client, gitHubImpl)
        msixBundleDetection(allManifestData)
    }

    private suspend fun Terminal.promptForInstaller(client: HttpClient): Url {
        println(colors.brightGreen(installerUrlInfo))
        return prompt(installerUrlConst) { input ->
            runBlocking { isUrlValid(url = Url(input), canBeBlank = false, client) }
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(Url(input.trim()))
        }?.let {
            println()
            promptIfRedirectedUrl(it, client)
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    private suspend fun Terminal.promptIfRedirectedUrl(installerUrl: Url, client: HttpClient): Url {
        val redirectedUrl = installerUrl.getRedirectedUrl(client)
        return if (
            redirectedUrl != installerUrl &&
            !installerUrl.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true)
        ) {
            println(colors.brightYellow(redirectFound))
            println(colors.cyan("Discovered URL: $redirectedUrl"))
            if (yesNoMenu(default = true)) {
                val error = isUrlValid(url = redirectedUrl, canBeBlank = false, client)
                if (error == null) {
                    success("URL changed to $redirectedUrl")
                } else {
                    warning(error)
                    warning(detectedUrlValidationFailed)
                    return installerUrl
                }
                println()
            } else {
                info("Original URL Retained - Proceeding with $installerUrl")
            }
            redirectedUrl
        } else {
            installerUrl
        }
    }

    private suspend fun Terminal.downloadInstaller(
        allManifestData: AllManifestData,
        client: HttpClient,
        gitHubImpl: GitHubImpl
    ) = with(allManifestData) {
        if (installers.map { it.installerUrl }.contains(installerUrl)) {
            installers += installers.first { it.installerUrl == installerUrl }
            skipAddInstaller = true
        } else {
            if (installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                gitHubDetection = GitHubDetection(installerUrl, gitHubImpl, client)
            } else {
                pageScraper = PageScraper(installerUrl, client)
            }
            val progress = getDownloadProgressBar(installerUrl).apply(ProgressAnimation::start)
            val downloadedFile = client.downloadFile(installerUrl, packageIdentifier, packageVersion, progress)
            val fileAnalyser = FileAnalyser(downloadedFile.file)
            installerType = fileAnalyser.getInstallerType()
            architecture = installerUrl.findArchitecture() ?: fileAnalyser.getArchitecture()
            scope = fileAnalyser.getScope()
            upgradeBehavior = fileAnalyser.getUpgradeBehaviour()
            installerSha256 = gitHubDetection?.sha256?.await() ?: downloadedFile.file.hash()
            when (downloadedFile.file.extension.lowercase()) {
                InstallerManifest.InstallerType.MSIX.toString(),
                InstallerManifest.InstallerType.APPX.toString() -> msix = Msix(downloadedFile.file)
                MsixBundle.msixBundleConst,
                MsixBundle.appxBundleConst -> msixBundle = MsixBundle(downloadedFile.file)
                InstallerManifest.InstallerType.MSI.toString() -> if (Platform.isWindows()) msi = Msi(downloadedFile.file)
                InstallerManifest.InstallerType.ZIP.toString() -> zip = Zip(
                    zip = downloadedFile.file,
                    terminal = this@downloadInstaller
                )
            }
            downloadedFile.file.delete()
            Runtime.getRuntime().removeShutdownHook(downloadedFile.fileDeletionThread)
        }
    }

    private fun Terminal.msixBundleDetection(allManifestData: AllManifestData) = with(allManifestData) {
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
                                cell(colors.brightWhite("${" ".repeat(Prompts.optionIndent)} $newText: $newValue"))
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

    suspend fun isUrlValid(url: Url?, canBeBlank: Boolean, client: HttpClient): String? {
        return when {
            url == null -> null
            url == Url(URLBuilder()) && canBeBlank -> null
            url == Url(URLBuilder()) -> Errors.blankInput(installerUrlConst)
            url.toString().length > maxLength -> Errors.invalidLength(max = maxLength)
            !url.toString().matches(regex) -> Errors.invalidRegex(regex)
            else -> client.checkUrlResponse(url)
        }
    }

    private suspend fun HttpClient.checkUrlResponse(url: Url): String? {
        return config { followRedirects = false }.use {
            try {
                val installerUrlResponse = it.head(url)
                if (!installerUrlResponse.status.isSuccess() && !installerUrlResponse.status.isRedirect()) {
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

    private const val redirectFound = "The URL appears to be redirected. " +
        "Would you like to use the destination URL instead?"

    private const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

    private const val installerUrlConst = "Installer Url"

    private const val maxLength = 2048
    private const val pattern = "^([Hh][Tt][Tt][Pp][Ss]?)://.+$"
    private val regex = Regex(pattern)
}
