import Hashing.hash
import com.appmattus.crypto.Algorithm
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.onDownload
import io.ktor.client.request.get
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpHeaders
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import org.apache.commons.io.FilenameUtils
import schemas.Patterns
import schemas.Schema
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

class NewManifest(private val terminal: Terminal, schemas: List<Schema?>) {
    private var packageVersion: String? = null
    private var installerUrl: String? = null
    private var packageIdentifier: String? = null
    private var installerHash: String? = null
    private val patterns = Patterns(schemas)

    private val client = HttpClient(CIO) {
        install(UserAgent) {
            agent = "Microsoft-Delivery-Optimization/10.1"
        }
        followRedirects = false
    }

    suspend fun main() {
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
            val lengthValid = identifierLength > Patterns.packageIdentifierMinLength && identifierLength < patterns.packageIdentifierMaxLength
            val identifierValid = packageIdentifier?.matches(patterns.packageIdentifier) ?: false
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
        var packageVersionSuccessful = false
        while (!packageVersionSuccessful) {
            println(brightGreen("[Required] Enter the version. For example: 1.33.7"))
            packageVersion = prompt(brightWhite("Package Version"))?.trim()
            val isLessThanMax = (packageVersion?.length ?: 0) < patterns.packageIdentifierMaxLength
            val versionValid = packageVersion?.matches(patterns.packageVersion) ?: false
            when {
                versionValid && isLessThanMax -> packageVersionSuccessful = true
                !isLessThanMax -> println(red(Errors.invalidLength(min = 1, max = 128)))
                !versionValid -> println(red(Errors.invalidRegex))
                else -> println(red(Errors.genericError))
            }
            println()
        }
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        while (installerUrl.isNullOrBlank()) {
            println(brightGreen("[Required] Enter the download url to the installer."))
            installerUrl = prompt(brightWhite("Url"))?.trim()
        }

        val redirectedUrl = client.getRedirectedUrl(installerUrl!!)
        var shouldUseRedirectedUrl = false
        if (redirectedUrl != installerUrl) {
            println(yellow("The URL appears to be redirected. Would you like to use the destination URL instead?"))
            println(blue("Discovered URL: $redirectedUrl"))
            println(brightGreen("   [Y] Use detected URL"))
            println(brightWhite("   [N] Use original URL"))
            val response: String? = prompt("Enter Choice (default is 'Y')")
            shouldUseRedirectedUrl = response != "N".lowercase()
        }
        if (shouldUseRedirectedUrl) {
            installerUrl = redirectedUrl
            println(yellow("[Warning] URL Changed - The URL was changed during processing and will be re-validated"))
        } else {
            println(brightGreen("Original URL Retained - Proceeding with $installerUrl"))
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
        val httpResponse: HttpResponse = client.get(installerUrl!!) {
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

    private suspend fun HttpClient.getRedirectedUrl(installerUrl: String): String {
        val response = head(installerUrl)
        var redirectedInstallerUrl: String = installerUrl

        var status = response.status.value
        var location = response.headers["Location"]
        while (status in 301..308 && response.headers.contains(HttpHeaders.Location) && location != null) {
            redirectedInstallerUrl = location
            val newResponse = head(redirectedInstallerUrl)
            status = newResponse.status.value
            location = newResponse.headers["Location"]
        }
        return redirectedInstallerUrl
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) {
            urlExtension = "winget-tmp"
        }
        return urlExtension
    }

}