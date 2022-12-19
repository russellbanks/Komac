package data

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
import schemas.InstallerSchema
import schemas.SchemasImpl
import java.io.File

object InstallerUrl : KoinComponent {
    suspend fun Terminal.installerDownloadPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightGreen(Prompts.installerUrlInfo))
            val input = prompt(brightWhite(PromptType.InstallerUrl.toString()))?.trim()
            val (installerUrlValid, error) = isInstallerUrlValid(input)
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
            println(brightYellow(Prompts.Redirection.redirectFound))
            println(cyan(Prompts.Redirection.discoveredUrl(redirectedUrl)))
            println((brightGreen(Prompts.Redirection.useDetectedUrl)))
            println(brightWhite(Prompts.Redirection.useOriginalUrl))
            if (prompt(Prompts.enterChoice, default = "Y")?.trim()?.lowercase() != "N".lowercase()) {
                println(brightYellow(Prompts.Redirection.urlChanged))
                val (redirectedUrlValid, error) = isInstallerUrlValid(redirectedUrl)
                error?.let { println(it) }
                if (redirectedUrlValid == Validation.Success) {
                    installerManifestData.installerUrl = redirectedUrl.toString()
                } else {
                    println()
                    println(brightYellow(Prompts.Redirection.detectedUrlValidationFailed))
                }
                println()
            } else {
                println(brightGreen(Prompts.Redirection.originalUrlRetained(installerManifestData.installerUrl)))
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

    suspend fun isInstallerUrlValid(
        url: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerUrlSchema = installerSchema.definitions.installer.properties.installerUrl
        return when {
            url.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerUrl)
            url.length > installerUrlSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerUrlSchema.maxLength)
            }
            !url.matches(Regex(installerUrlSchema.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(installerUrlSchema.pattern))
            }
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
}
