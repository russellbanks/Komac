import Hashing.hash
import Ktor.isRedirect
import com.appmattus.crypto.Algorithm
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.onDownload
import io.ktor.client.request.get
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpHeaders
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import org.apache.commons.io.FilenameUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.InstallerSchemaImpl
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private var packageVersion: String? = null
    private var installerUrl: String? = null
    private var packageIdentifier: String? = null
    private var installerHash: String? = null
    private var architecture: String? = null
    private val installerSchemaImpl: InstallerSchemaImpl = get()

    private val client = HttpClient(Java) {
        install(UserAgent) {
            agent = "Microsoft-Delivery-Optimization/10.1"
        }
        followRedirects = false
    }

    suspend fun main() {
        while (installerSchemaImpl.installerSchema == null) delay(1)
        with(terminal) {
            packageIdentifierPrompt()
            packageVersionPrompt()
            installerDownloadPrompt()
            architecturePrompt()
        }
    }

    private fun Terminal.packageIdentifierPrompt() {
        do {
            println(brightGreen(Prompts.packageIdentifierInfo))
            packageIdentifier = prompt(brightWhite(Prompts.packageIdentifier))?.trim()
            val packageIdentifierValid = installerSchemaImpl.isPackageIdentifierValid(packageIdentifier)
            when (packageIdentifierValid) {
                Validation.InvalidLength -> {
                    println(red(Errors.invalidLength(
                        min = InstallerSchemaImpl.packageIdentifierMinLength,
                        max = installerSchemaImpl.packageIdentifierMaxLength
                    )))
                }
                Validation.InvalidPattern -> {
                    println(red(Errors.invalidRegex(installerSchemaImpl.packageIdentifierPattern)))
                }
                else -> { /* Success */ }
            }
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    private fun Terminal.packageVersionPrompt() {
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            packageVersion = prompt(brightWhite(Prompts.packageVersion))?.trim()
            val packageVersionValid = installerSchemaImpl.isPackageVersionValid(packageVersion)
            when (packageVersionValid) {
                Validation.Blank -> println(red(Errors.blankInput(PromptType.PackageVersion)))
                Validation.InvalidLength -> {
                    println(red(Errors.invalidLength(max = installerSchemaImpl.packageVersionMaxLength)))
                }
                Validation.InvalidPattern -> {
                    println(red(Errors.invalidRegex(installerSchemaImpl.packageVersionPattern)))
                }
                else -> { /* Success */ }
            }
            println()
        } while (packageVersionValid != Validation.Success)
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        var installerUrlResponse: HttpResponse? = null
        do {
            println(brightGreen(Prompts.installerUrlInfo))
            installerUrl = prompt(brightWhite(Prompts.installerUrl))?.trim()
            val installerUrlValid = installerSchemaImpl.isInstallerUrlValid(installerUrl) {
                runCatching { installerUrlResponse = client.head(installerUrl!!) }
                installerUrlResponse
            }
            when (installerUrlValid) {
                Validation.Blank -> println(red(Errors.blankInput(PromptType.InstallerUrl)))
                Validation.InvalidLength -> {
                    println(red(Errors.invalidLength(max = installerSchemaImpl.installerUrlMaxLength)))
                }
                Validation.InvalidPattern -> println(red(Errors.invalidRegex(installerSchemaImpl.installerUrlPattern)))
                Validation.UnsuccessfulResponseCode -> {
                    println(red(Errors.unsuccessfulUrlResponse(installerUrlResponse)))
                }
                else -> { /* Success */ }
            }
            println()
        } while (installerUrlValid != Validation.Success)

        val redirectedUrl = client.getRedirectedUrl(installerUrl, installerUrlResponse)
        if (redirectedUrl != installerUrl && redirectedUrl?.contains(other = "github", ignoreCase = true) != true) {
            println(yellow(Prompts.Redirection.redirectFound))
            println(blue(Prompts.Redirection.discoveredUrl(redirectedUrl)))
            println((brightGreen(Prompts.Redirection.useDetectedUrl)))
            println(brightWhite(Prompts.Redirection.useOriginalUrl))
            if (prompt(Prompts.Redirection.enterChoice, default = "Y")?.lowercase() != "N".lowercase()) {
                installerUrl = redirectedUrl
                println(yellow(Prompts.Redirection.urlChanged))
            } else {
                println(brightGreen(Prompts.Redirection.originalUrlRetained(installerUrl)))
            }
        }

        val progress = progressAnimation {
            text(FilenameUtils.getName(installerUrl))
            percentage()
            progressBar()
            completed()
            speed("B/s")
            timeRemaining()
        }

        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                /* prefix = */ "$packageIdentifier v$packageVersion - $formattedDate",
                /* suffix = */ ".${getURLExtension(installerUrl)}"
            )
        }

        progress.start()
        val httpResponse: HttpResponse = client.get(installerUrl as String) {
            onDownload { bytesSentTotal, contentLength ->
                progress.update(bytesSentTotal, contentLength)
            }
        }
        progress.stop()
        progress.clear()
        client.close()
        val responseBody: ByteArray = httpResponse.body()
        file.writeBytes(responseBody)
        installerHash = file.hash(Algorithm.SHA_256).uppercase()

        println("Sha256: $installerHash")
        file.delete()
    }

    private fun Terminal.architecturePrompt() {
        do {
            println(brightGreen(Prompts.architectureInfo(installerSchemaImpl)))
            architecture = prompt(brightWhite(Prompts.architecture))?.trim()?.lowercase()
            val architectureValid = installerSchemaImpl.isArchitectureValid(architecture)
            when (architectureValid) {
                Validation.Blank -> println(red(Errors.blankInput(PromptType.Architecture)))
                Validation.InvalidArchitecture -> println(red(Errors.invalidArchitecture(installerSchemaImpl)))
                else -> { /* Success */ }
            }
            println()
        } while (architectureValid != Validation.Success)
    }

    private suspend fun HttpClient.getRedirectedUrl(installerUrl: String?, httpResponse: HttpResponse?): String? {
        var redirectedInstallerUrl: String? = installerUrl

        var status = httpResponse?.status
        var location = httpResponse?.headers?.get("Location")
        while (
            status?.isRedirect() == true &&
            httpResponse?.headers?.contains(HttpHeaders.Location) == true &&
            location != null
        ) {
            redirectedInstallerUrl = location
            val newResponse = head(redirectedInstallerUrl)
            status = newResponse.status
            location = newResponse.headers["Location"]
        }
        return redirectedInstallerUrl
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) urlExtension = "winget-tmp"
        return urlExtension
    }
}
