import Hashing.hash
import com.appmattus.crypto.Algorithm
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.onDownload
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import org.apache.commons.io.FilenameUtils
import schemas.ManifestVersionSchema
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

class NewManifest(private val terminal: Terminal, private val manifestVersionSchema: ManifestVersionSchema) {
    var packageVersion: String? = null
    var installerUrl: String? = null
    var packageIdentifier: String? = null
    var installerHash: String? = null
    val client = HttpClient(CIO) {
        install(ContentNegotiation) {
            json(
                Json {
                    ignoreUnknownKeys = true
                }
            )
        }
    }

    suspend fun run() {
        with(terminal) {
            packageIdentifierPrompt()
            println()
            packageVersionPrompt()
            println()
            installerDownloadPrompt()
        }
    }

    private fun Terminal.packageIdentifierPrompt() {
        var packageIdentifierSuccessful = false
        println(brightGreen("[Required] Enter the Package Identifier, in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"))
        packageIdentifier = prompt(brightWhite("Package Identifier"))?.trim()
        while(!packageIdentifierSuccessful) {
            if ((packageIdentifier?.length ?: 0) < 4) {
                println(red(Errors.invalidLength(min = 4, max = 128)))
                println()
                println(brightGreen("[Required] Enter the Package Identifier, in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"))
                packageIdentifier = prompt(brightWhite("Package Identifier"))?.trim()
            } else {
                packageIdentifierSuccessful = true
            }
        }
    }

    private fun Terminal.packageVersionPrompt() {
        var packageVersionSuccessful = false
        println(brightGreen("[Required] Enter the version. For example: 1.33.7"))
        packageVersion = prompt(brightWhite("Package Version"))?.trim()
        val lessThanMax = (packageVersion?.length ?: 0) > manifestVersionSchema.properties.packageVersion.maxLength
        while(!packageVersionSuccessful) {
            if (lessThanMax || packageVersion?.isNotBlank() != true) {
                packageVersionError(Errors.invalidLength(min = 1, max = 128))
            } else if (packageVersion?.matches(manifestVersionSchema.properties.packageVersion.pattern.toRegex()) != true) {
                packageVersionError(Errors.invalidRegex)
            } else {
                packageVersionSuccessful = true
            }
        }
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        println(brightGreen("[Required] Enter the download url to the installer."))
        installerUrl = prompt(brightWhite("Url"))?.trim()
        val progress = progressAnimation {
            text("my-file.iso")
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
        val responseBody: ByteArray = httpResponse.body()
        file.writeBytes(responseBody)
        installerHash = file.hash(Algorithm.SHA_256).uppercase()

        println("A file saved to ${file.path}")
        file.delete()
    }

    private fun Terminal.packageVersionError(error: String) {
        println(red(error))
        println()
        println(brightGreen("[Required] Enter the version. For example: 1.33.7"))
        packageVersion = prompt(brightWhite("Package Version"))?.trim()
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) {
            urlExtension = "winget-tmp"
        }
        return urlExtension
    }

}