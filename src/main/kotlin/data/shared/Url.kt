package data.shared

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.sun.jna.Platform
import data.AllManifestData
import detection.PageScraper
import detection.files.Zip
import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import detection.github.GitHubDetection
import input.ExitCode
import input.Prompts
import io.ktor.client.network.sockets.ConnectTimeoutException
import io.ktor.client.request.head
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import io.ktor.http.isSuccess
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.findArchitecture
import network.getRedirectedUrl
import network.isRedirect
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import utils.FileAnalyser
import utils.Hashing.hash
import java.net.ConnectException
import kotlin.system.exitProcess

object Url : KoinComponent {
    private val allManifestData: AllManifestData by inject()

    suspend fun Terminal.installerDownloadPrompt(parameterUrl: Url? = null) = with(allManifestData) {
        if (parameterUrl != null) {
            installerUrl = parameterUrl
        } else {
            setInstallerUrlFromPrompt(this)
        }
        downloadInstaller()
        msixBundleDetection()
    }

    private suspend fun Terminal.setInstallerUrlFromPrompt(allManifestData: AllManifestData) {
        println(colors.brightGreen(installerUrlInfo))
        allManifestData.installerUrl = prompt(
            prompt = installerUrlConst,
            convert = { input ->
                runBlocking { isUrlValid(url = Url(input), canBeBlank = false) }
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(Url(input.trim()))
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()

        setRedirectedUrl(allManifestData)
    }

    private suspend fun Terminal.setRedirectedUrl(allManifestData: AllManifestData) {
        val redirectedUrl = allManifestData.installerUrl.getRedirectedUrl(get<Http>().client)
        if (
            redirectedUrl != allManifestData.installerUrl &&
            redirectedUrl.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true)
        ) {
            println(
                verticalLayout {
                    cell(colors.brightYellow(redirectFound))
                    cell(colors.cyan("Discovered URL: $redirectedUrl"))
                    cell(colors.brightGreen(useDetectedUrl))
                    cell(colors.brightWhite(useOriginalUrl))
                }
            )
            if (prompt(prompt = Prompts.enterChoice, default = "Y")?.trim()?.lowercase() != "N".lowercase()) {
                warning(urlChanged)
                val error = isUrlValid(url = redirectedUrl, canBeBlank = false)
                if (error == null) {
                    allManifestData.installerUrl = redirectedUrl
                    success("URL changed to $redirectedUrl")
                } else {
                    warning(error)
                    warning(detectedUrlValidationFailed)
                }
                println()
            } else {
                info("Original URL Retained - Proceeding with ${allManifestData.installerUrl}")
            }
        }
    }

    private suspend fun Terminal.downloadInstaller() = with(allManifestData) {
        if (installers.map { it.installerUrl }.contains(installerUrl)) {
            installers += installers.first { it.installerUrl == installerUrl }
            skipAddInstaller = true
        } else {
            if (installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                gitHubDetection = GitHubDetection(installerUrl)
            } else {
                pageScraper = PageScraper(installerUrl)
            }
            val (file, fileDeletionThread) = get<Http>().client.downloadFile(installerUrl, this@downloadInstaller)
            val fileAnalyser = FileAnalyser(file)
            installerType = fileAnalyser.getInstallerType()
            architecture = installerUrl.findArchitecture() ?: fileAnalyser.getArchitecture()
            scope = fileAnalyser.getScope()
            upgradeBehavior = fileAnalyser.getUpgradeBehaviour()
            installerSha256 = gitHubDetection?.sha256?.await() ?: file.hash()
            when (file.extension.lowercase()) {
                InstallerManifest.InstallerType.MSIX.toString(),
                InstallerManifest.InstallerType.APPX.toString() -> msix = Msix(file)
                MsixBundle.msixBundleConst,
                MsixBundle.appxBundleConst -> msixBundle = MsixBundle(file)
                InstallerManifest.InstallerType.MSI.toString() -> if (Platform.isWindows()) msi = Msi(file)
                InstallerManifest.InstallerType.ZIP.toString() -> zip = Zip(
                    zip = file,
                    terminal = this@downloadInstaller
                )
            }
            file.delete()
            Runtime.getRuntime().removeShutdownHook(fileDeletionThread)
        }
    }

    private fun Terminal.msixBundleDetection() = with(allManifestData) {
        if (msixBundle != null) {
            println(
                verticalLayout {
                    cell(
                        (colors.brightGreen + colors.bold)(
                            "${msixBundle?.packages?.size} packages have been detected inside the MSIX Bundle:"
                        )
                    )
                    msixBundle?.packages?.forEachIndexed { index, individualPackage ->
                        cell(brightGreen("Package ${index.inc()}/${msixBundle?.packages?.size}"))
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
                                    newValue = value.joinToString(", ")
                                }
                                cell(brightWhite("${" ".repeat(Prompts.optionIndent)} $newText: $newValue"))
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

    suspend fun isUrlValid(url: Url?, canBeBlank: Boolean): String? {
        return when {
            url == null -> null
            url == Url(URLBuilder()) && canBeBlank -> null
            url == Url(URLBuilder()) -> Errors.blankInput(installerUrlConst)
            url.toString().length > maxLength -> Errors.invalidLength(max = maxLength)
            !url.toString().matches(regex) -> Errors.invalidRegex(regex)
            else -> checkUrlResponse(url)
        }
    }

    private suspend fun checkUrlResponse(url: Url): String? {
        return get<Http>().client.config { followRedirects = false }.use {
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

    private const val useDetectedUrl = "   [Y] Use detected URL"

    private const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

    private const val useOriginalUrl = "   [N] Use original URL"

    private const val urlChanged = "[Warning] URL Changed - " +
        "The URL was changed during processing and will be re-validated"

    private const val installerUrlConst = "Installer Url"

    private const val maxLength = 2048
    private const val pattern = "^([Hh][Tt][Tt][Pp][Ss]?)://.+$"
    private val regex = Regex(pattern)
}
