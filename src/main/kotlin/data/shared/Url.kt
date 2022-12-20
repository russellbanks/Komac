package data.shared

import Errors
import Ktor
import Ktor.downloadInstallerFromUrl
import Ktor.getRedirectedUrl
import Ktor.isRedirect
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.InstallerManifestData
import data.locale.LocaleUrl
import hashing.Hashing
import hashing.Hashing.hash
import input.PromptType
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import io.ktor.http.isSuccess
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.InstallerSchema
import schemas.RemoteSchema
import schemas.SchemasImpl
import java.io.File

object Url : KoinComponent {
    suspend fun Terminal.installerDownloadPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        do {
            println(brightGreen(installerUrlInfo))
            val input = prompt(brightWhite(PromptType.InstallerUrl.toString()))?.trim()
            val (installerUrlValid, error) = isUrlValid(
                url = input,
                schema = schemasImpl.installerSchema,
                canBeBlank = false
            )
            if (installerUrlValid == Validation.Success && input != null) {
                installerManifestData.installerUrl = input
            }
            error?.let { println(red(it)) }
            println()
        } while (installerUrlValid != Validation.Success)

        val redirectedUrl = getRedirectedUrl(installerManifestData.installerUrl)
        if (
            redirectedUrl != installerManifestData.installerUrl &&
            redirectedUrl?.contains(other = "github", ignoreCase = true) != true
        ) {
            println(brightYellow(redirectFound))
            println(cyan(discoveredUrl(redirectedUrl)))
            println((brightGreen(useDetectedUrl)))
            println(brightWhite(useOriginalUrl))
            if (prompt(Prompts.enterChoice, default = "Y")?.trim()?.lowercase() != "N".lowercase()) {
                println(brightYellow(urlChanged))
                val (redirectedUrlValid, error) = isUrlValid(
                    url = redirectedUrl,
                    schema = schemasImpl.installerSchema,
                    canBeBlank = false
                )
                error?.let { println(it) }
                if (redirectedUrlValid == Validation.Success) {
                    installerManifestData.installerUrl = redirectedUrl.toString()
                } else {
                    println()
                    println(brightYellow(detectedUrlValidationFailed))
                }
                println()
            } else {
                println(brightGreen(originalUrlRetained(installerManifestData.installerUrl)))
            }
        }

        lateinit var downloadedFile: File
        HttpClient(Java) {
            install(UserAgent) {
                agent = Ktor.userAgent
            }
        }.use { downloadedFile = it.downloadInstallerFromUrl() }
        installerManifestData.installerSha256 = downloadedFile.hash(Hashing.Algorithms.SHA256).uppercase()
        downloadedFile.delete()
        println("Sha256: ${installerManifestData.installerSha256}")
    }

    suspend fun Terminal.localeUrlPrompt(localeUrl: LocaleUrl) {
        val defaultLocaleSchema: DefaultLocaleSchema = get<SchemasImpl>().defaultLocaleSchema
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        do {
            println(brightYellow(publisherUrlInfo(localeUrl, defaultLocaleSchema)))
            val input = prompt(brightWhite(localeUrl.toString()))?.trim()
            val (publisherUrlValid, error) = isUrlValid(url = input, schema = defaultLocaleSchema, canBeBlank = true)
            if (publisherUrlValid == Validation.Success) {
                when (localeUrl) {
                    LocaleUrl.PackageUrl -> defaultLocaleManifestData.packageUrl = input
                    LocaleUrl.PublisherUrl -> defaultLocaleManifestData.publisherUrl = input
                    LocaleUrl.PublisherSupportUrl -> defaultLocaleManifestData.publisherSupportUrl = input
                    LocaleUrl.PublisherPrivacyUrl -> defaultLocaleManifestData.publisherPrivacyUrl = input
                }
            }
            error?.let { println(red(it)) }
            println()
        } while (publisherUrlValid != Validation.Success)
    }

    suspend fun isUrlValid(url: String?, schema: RemoteSchema, canBeBlank: Boolean): Pair<Validation, String?> {
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
            url.isNullOrBlank() && canBeBlank -> Validation.Success to null
            url.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerUrl)
            url.length > maxLength -> Validation.InvalidLength to Errors.invalidLength(max = maxLength)
            !url.matches(pattern) -> Validation.InvalidPattern to Errors.invalidRegex(pattern)
            else -> {
                lateinit var installerUrlResponse: HttpResponse
                HttpClient(Java) {
                    install(UserAgent) {
                        agent = Ktor.userAgent
                    }
                    followRedirects = false
                }.use { installerUrlResponse = it.head(url) }
                if (!installerUrlResponse.status.isSuccess() && !installerUrlResponse.status.isRedirect()) {
                    Validation.UnsuccessfulResponseCode to Errors.unsuccessfulUrlResponse(installerUrlResponse)
                } else {
                    Validation.Success to null
                }
            }
        }
    }

    private fun publisherUrlInfo(publisherUrl: LocaleUrl, defaultLocaleSchema: DefaultLocaleSchema): String {
        val description = when (publisherUrl) {
            LocaleUrl.PackageUrl -> defaultLocaleSchema.properties.packageUrl.description
            LocaleUrl.PublisherUrl -> defaultLocaleSchema.properties.publisherUrl.description
            LocaleUrl.PublisherSupportUrl -> defaultLocaleSchema.properties.publisherSupportUrl.description
            LocaleUrl.PublisherPrivacyUrl -> defaultLocaleSchema.properties.privacyUrl.description
        }
        return "${Prompts.optional} Enter ${description.lowercase()}"
    }

    private fun originalUrlRetained(url: String?) = "Original URL Retained - Proceeding with $url"

    private fun discoveredUrl(url: String?) = "Discovered URL: $url"

    private const val installerUrlInfo = "${Prompts.required} Enter the download url to the installer."

    private const val redirectFound = "The URL appears to be redirected. " +
        "Would you like to use the destination URL instead?"

    private const val useDetectedUrl = "   [Y] Use detected URL"

    private const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

    private const val useOriginalUrl = "   [N] Use original URL"

    private const val urlChanged = "[Warning] URL Changed - " +
        "The URL was changed during processing and will be re-validated"
}
