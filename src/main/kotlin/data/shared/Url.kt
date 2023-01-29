package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.sun.jna.Platform
import data.DefaultLocaleManifestData
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.locale.LocaleUrl
import detection.GitHubDetection
import detection.files.Zip
import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import hashing.Hashing.hash
import input.Prompts
import io.ktor.client.network.sockets.ConnectTimeoutException
import io.ktor.client.request.head
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import io.ktor.http.isSuccess
import kotlinx.coroutines.runBlocking
import ktor.Http
import ktor.Ktor.decodeHex
import ktor.Ktor.downloadInstallerFromUrl
import ktor.Ktor.getRedirectedUrl
import ktor.Ktor.getURLExtension
import ktor.Ktor.isRedirect
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema
import schemas.data.InstallerSchema
import schemas.data.RemoteSchema
import schemas.manifest.InstallerManifest
import java.net.ConnectException
import kotlin.system.exitProcess

object Url : KoinComponent {
    private val schemasImpl: SchemasImpl by inject()

    suspend fun Terminal.installerDownloadPrompt(parameterUrl: Url? = null) {
        val installerManifestData: InstallerManifestData by inject()
        if (parameterUrl != null) {
            installerManifestData.installerUrl = parameterUrl
        } else {
            setInstallerUrlFromPrompt(installerManifestData)
        }
        downloadInstaller(installerManifestData)
        msixBundleDetection()
    }

    private suspend fun Terminal.setInstallerUrlFromPrompt(installerManifestData: InstallerManifestData) {
        println(colors.brightGreen(installerUrlInfo))
        installerManifestData.installerUrl = prompt(
            prompt = installerUrlConst,
            convert = { input ->
                runBlocking { isUrlValid(url = Url(input), schema = schemasImpl.installerSchema, canBeBlank = false) }
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(Url(input.trim()).decodeHex())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()

        setRedirectedUrl(installerManifestData)
    }

    private suspend fun Terminal.setRedirectedUrl(installerManifestData: InstallerManifestData) {
        val redirectedUrl = getRedirectedUrl(installerManifestData.installerUrl)
        if (
            redirectedUrl != installerManifestData.installerUrl &&
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
                val error = isUrlValid(url = redirectedUrl, schema = schemasImpl.installerSchema, canBeBlank = false)
                if (error == null) {
                    installerManifestData.installerUrl = redirectedUrl
                    success("URL changed to $redirectedUrl")
                } else {
                    warning(error)
                    warning(detectedUrlValidationFailed)
                }
                println()
            } else {
                info("Original URL Retained - Proceeding with ${installerManifestData.installerUrl}")
            }
        }
    }

    private suspend fun Terminal.downloadInstaller(installerManifestData: InstallerManifestData) {
        val sharedManifestData: SharedManifestData by inject()
        if (installerManifestData.installers.map { it.installerUrl }.contains(installerManifestData.installerUrl)) {
            val storedInstaller = installerManifestData.installers.first {
                it.installerUrl == installerManifestData.installerUrl
            }
            with(installerManifestData) {
                installerSha256 = storedInstaller.installerSha256
                productCode = storedInstaller.productCode
            }
        } else {
            if (installerManifestData.installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                sharedManifestData.gitHubDetection = GitHubDetection(installerManifestData.installerUrl)
            }
            if (sharedManifestData.gitHubDetection?.sha256?.await() == null &&
                getURLExtension(installerManifestData.installerUrl).equals(
                    other = InstallerManifest.InstallerType.EXE.toString(),
                    ignoreCase = true
                )
            ) {
                val (file, fileDeletionThread) = get<Http>().client.downloadInstallerFromUrl(terminal = this)
                installerManifestData.installerSha256 = file.hash()
                when (file.extension.lowercase()) {
                    InstallerManifest.InstallerType.MSIX.toString(),
                    InstallerManifest.InstallerType.APPX.toString() -> sharedManifestData.msix = Msix(file)
                    MsixBundle.msixBundleConst,
                    MsixBundle.appxBundleConst -> sharedManifestData.msixBundle = MsixBundle(file)
                    InstallerManifest.InstallerType.MSI.toString() -> {
                        if (Platform.isWindows()) sharedManifestData.msi = Msi(file)
                    }
                    InstallerManifest.InstallerType.ZIP.toString() -> sharedManifestData.zip = Zip(
                        zip = file,
                        terminal = this@downloadInstaller
                    )
                }
                file.delete()
                Runtime.getRuntime().removeShutdownHook(fileDeletionThread)
            } else {
                sharedManifestData.gitHubDetection?.sha256?.await()?.let { installerManifestData.installerSha256 = it }
            }
        }
    }

    private fun Terminal.msixBundleDetection() {
        val msixBundle = get<SharedManifestData>().msixBundle
        if (msixBundle != null) {
            println(
                verticalLayout {
                    cell(
                        (colors.brightGreen + colors.bold)(
                            "${msixBundle.packages?.size} packages have been detected inside the MSIX Bundle:"
                        )
                    )
                    msixBundle.packages?.forEachIndexed { index, individualPackage ->
                        cell(brightGreen("Package ${index.inc()}/${msixBundle.packages?.size}"))
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

    suspend fun Terminal.localeUrlPrompt(localeUrl: LocaleUrl) {
        val defaultLocaleSchema: DefaultLocaleSchema = get<SchemasImpl>().defaultLocaleSchema
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val gitHubDetection: GitHubDetection? = get<SharedManifestData>().gitHubDetection
        when {
            gitHubDetection?.licenseUrl != null && localeUrl == LocaleUrl.LicenseUrl -> {
                defaultLocaleManifestData.licenseUrl = gitHubDetection.licenseUrl.await()
            }
            gitHubDetection?.publisherUrl != null && localeUrl == LocaleUrl.PublisherUrl -> {
                defaultLocaleManifestData.publisherUrl = gitHubDetection.publisherUrl.await()
            }
            gitHubDetection?.releaseNotesUrl != null && localeUrl == LocaleUrl.ReleaseNotesUrl -> {
                defaultLocaleManifestData.releaseNotesUrl = gitHubDetection.releaseNotesUrl.await()
            }
            gitHubDetection?.packageUrl != null && localeUrl == LocaleUrl.PackageUrl -> {
                defaultLocaleManifestData.packageUrl = gitHubDetection.packageUrl.await()
            }
            gitHubDetection?.publisherSupportUrl != null && localeUrl == LocaleUrl.PublisherSupportUrl -> {
                defaultLocaleManifestData.publisherSupportUrl = gitHubDetection.publisherSupportUrl.await()
            }
            else -> {
                println(colors.brightYellow(localeUrlInfo(localeUrl, defaultLocaleSchema.properties)))
                val input = prompt(
                    prompt = localeUrl.toString(),
                    default = getPreviousValue(localeUrl)?.also { muted("Previous $localeUrl: $it") },
                    convert = { input ->
                        runBlocking {
                            isUrlValid(url = Url(input.trim()), schema = defaultLocaleSchema, canBeBlank = true)
                        }?.let { ConversionResult.Invalid(it) }
                            ?: ConversionResult.Valid(input.trim().ifBlank { null }?.let { Url(it) })
                    }
                )
                when (localeUrl) {
                    LocaleUrl.CopyrightUrl -> defaultLocaleManifestData.copyrightUrl = input
                    LocaleUrl.LicenseUrl -> defaultLocaleManifestData.licenseUrl = input
                    LocaleUrl.PackageUrl -> defaultLocaleManifestData.packageUrl = input
                    LocaleUrl.PublisherUrl -> defaultLocaleManifestData.publisherUrl = input
                    LocaleUrl.PublisherSupportUrl -> defaultLocaleManifestData.publisherSupportUrl = input
                    LocaleUrl.PublisherPrivacyUrl -> defaultLocaleManifestData.publisherPrivacyUrl = input
                    LocaleUrl.ReleaseNotesUrl -> defaultLocaleManifestData.releaseNotesUrl = input
                }
                println()
            }
        }
    }

    suspend fun areUrlsValid(urls: List<Url>?): String? {
        urls?.forEach {
            val error = isUrlValid(it, get<SchemasImpl>().installerSchema, false)
            error?.let { return error }
        }
        return null
    }

    suspend fun isUrlValid(url: Url, schema: RemoteSchema, canBeBlank: Boolean): String? {
        val maxLength = when (schema) {
            is InstallerSchema -> schema.definitions.url.maxLength
            is DefaultLocaleSchema -> schema.definitions.url.maxLength
            else -> 0
        }
        val pattern = Regex(
            when (schema) {
                is InstallerSchema -> schema.definitions.url.pattern
                is DefaultLocaleSchema -> schema.definitions.url.pattern
                else -> ""
            }
        )
        return when {
            url == Url(URLBuilder()) && canBeBlank -> null
            url == Url(URLBuilder()) -> Errors.blankInput(installerUrlConst)
            url.toString().length > maxLength -> Errors.invalidLength(max = maxLength)
            !url.toString().matches(pattern) -> Errors.invalidRegex(pattern)
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

    private fun getPreviousValue(localeUrl: LocaleUrl): Url? {
        val remoteDefaultLocaleData = get<PreviousManifestData>().remoteDefaultLocaleData
        return when (localeUrl) {
            LocaleUrl.CopyrightUrl -> remoteDefaultLocaleData?.copyrightUrl
            LocaleUrl.LicenseUrl -> remoteDefaultLocaleData?.licenseUrl
            LocaleUrl.PackageUrl -> remoteDefaultLocaleData?.packageUrl
            LocaleUrl.PublisherUrl -> remoteDefaultLocaleData?.publisherUrl
            LocaleUrl.PublisherSupportUrl -> remoteDefaultLocaleData?.publisherSupportUrl
            LocaleUrl.PublisherPrivacyUrl -> remoteDefaultLocaleData?.privacyUrl
            LocaleUrl.ReleaseNotesUrl -> remoteDefaultLocaleData?.releaseNotesUrl
        }
    }

    private fun localeUrlInfo(publisherUrl: LocaleUrl, schemaProperties: DefaultLocaleSchema.Properties): String {
        val description = when (publisherUrl) {
            LocaleUrl.CopyrightUrl -> schemaProperties.copyrightUrl.description
            LocaleUrl.LicenseUrl -> schemaProperties.licenseUrl.description
            LocaleUrl.PackageUrl -> schemaProperties.packageUrl.description
            LocaleUrl.PublisherUrl -> schemaProperties.publisherUrl.description
            LocaleUrl.PublisherSupportUrl -> schemaProperties.publisherSupportUrl.description
            LocaleUrl.PublisherPrivacyUrl -> schemaProperties.privacyUrl.description
            LocaleUrl.ReleaseNotesUrl -> schemaProperties.releaseNotesUrl.description
        }
        return "${Prompts.optional} Enter ${description.lowercase()}"
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
}
