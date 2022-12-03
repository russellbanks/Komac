import Hashing.hash
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
import io.ktor.http.HttpStatusCode
import io.ktor.http.isSuccess
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
        }
    }

    private fun Terminal.packageIdentifierPrompt() {
        var packageIdentifierSuccessful = false
        while (!packageIdentifierSuccessful) {
            println(brightGreen("[Required] Enter the Package Identifier, in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"))
            packageIdentifier = prompt(brightWhite("Package Identifier"))?.trim()
            val identifierLength = packageIdentifier?.length ?: 0
            val lengthValid = identifierLength > 4 && identifierLength < installerSchemaImpl.packageIdentifierMaxLength
            val identifierValid = packageIdentifier?.matches(installerSchemaImpl.packageIdentifierPattern) ?: false
            when {
                identifierValid && lengthValid -> packageIdentifierSuccessful = true
                !lengthValid -> println(red(Errors.invalidLength(min = 4, max = 128)))
                !identifierValid -> println(red(Errors.invalidRegex))
                else -> println(red(Errors.genericError))
            }
            println()
        }
    }

    private fun Terminal.packageVersionPrompt() {
        do {
            println(brightGreen("[Required] Enter the version. For example: 1.33.7"))
            packageVersion = prompt(brightWhite("Package Version"))?.trim()
            val isVersionValid = packageVersion?.matches(installerSchemaImpl.packageVersionPattern) == true
            val isVersionLessThanMaxLength = (packageVersion?.length ?: 0) < installerSchemaImpl.packageVersionMaxLength
            when {
                packageVersion.isNullOrBlank() -> println(red("[Error] Version cannot be blank"))
                !isVersionLessThanMaxLength -> println(red("[Error] Invalid Length - Length must be less than ${installerSchemaImpl.packageVersionMaxLength}"))
                !isVersionValid -> println(red(Errors.invalidRegex))
            }
            println()
        } while (packageVersion.isNullOrBlank() || !isVersionValid || !isVersionLessThanMaxLength)
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        var installerUrlResponse: HttpResponse? = null

        do {
            println(brightGreen("[Required] Enter the download url to the installer."))
            installerUrl = prompt(brightWhite("Url"))?.trim()
            var status: HttpStatusCode? = null
            when {
                installerUrl.isNullOrBlank() -> println(red("[Error] Url cannot be blank"))
                (installerUrl?.length ?: 0) > installerSchemaImpl.installerUrlMaxLength -> {
                    println(red("[Error] Invalid Length - Length must be less than ${installerSchemaImpl.installerUrlMaxLength}"))
                }
                installerUrl?.matches(installerSchemaImpl.installerUrlPattern) != true -> {
                    println(red(Errors.invalidRegex))
                }
                else -> {
                    runCatching { installerUrlResponse = client.head(installerUrl!!) }
                    status = installerUrlResponse?.status ?: HttpStatusCode.BadRequest
                    if (!status.isSuccess() && !status.isRedirect()) {
                        println(red(Errors.unsuccessfulUrlResponse(installerUrlResponse)))
                    }
                }
            }
            println()
        } while (status?.isSuccess() != true && status?.isRedirect() != true)

        val redirectedUrl = client.getRedirectedUrl(installerUrl, installerUrlResponse)
        if (redirectedUrl != installerUrl && redirectedUrl?.contains("github") != true) {
            println(yellow("The URL appears to be redirected. Would you like to use the destination URL instead?"))
            println(blue("Discovered URL: $redirectedUrl"))
            println(brightGreen("   [Y] Use detected URL"))
            println(brightWhite("   [N] Use original URL"))
            if (prompt("Enter Choice (default is 'Y')")?.lowercase() != "N".lowercase()) {
                installerUrl = redirectedUrl
                println(yellow("[Warning] URL Changed - The URL was changed during processing and will be re-validated"))
            } else {
                println(brightGreen("Original URL Retained - Proceeding with $installerUrl"))
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

        println("A file saved to ${file.path}")
        file.delete()
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

    private fun HttpStatusCode.isRedirect(): Boolean {
        return value in HttpStatusCode.MovedPermanently.value..HttpStatusCode.PermanentRedirect.value
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) {
            urlExtension = "winget-tmp"
        }
        return urlExtension
    }
}
