
import com.github.ajalt.mordant.animation.progressAnimation
import data.InstallerManifestData
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.prepareGet
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentLength
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.commons.io.FilenameUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.TerminalInstance
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object Ktor : KoinComponent {
    suspend fun HttpClient.downloadInstallerFromUrl(): File {
        val terminalInstance: TerminalInstance by inject()
        val installerManifestData: InstallerManifestData by inject()
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                "${installerManifestData.packageIdentifier} v${installerManifestData.packageVersion} - $formattedDate",
                ".${getURLExtension(installerManifestData.installerUrl)}"
            )
        }

        with(terminalInstance.terminal) {
            progressAnimation {
                text(FilenameUtils.getName(installerManifestData.installerUrl))
                percentage()
                progressBar()
                completed()
                speed("B/s")
                timeRemaining()
            }.run {
                start()
                config { followRedirects = true }.use { client ->
                    client.prepareGet(installerManifestData.installerUrl as String).execute { httpResponse ->
                        val channel: ByteReadChannel = httpResponse.body()
                        while (!channel.isClosedForRead) {
                            val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                            while (packet.isNotEmpty) {
                                file.appendBytes(packet.readBytes())
                                update(file.length(), httpResponse.contentLength())
                            }
                        }
                    }
                }
                stop()
                clear()
            }
        }
        return file
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) urlExtension = "winget-tmp"
        return urlExtension
    }

    fun HttpStatusCode.isRedirect(): Boolean {
        return value in HttpStatusCode.MovedPermanently.value..HttpStatusCode.PermanentRedirect.value
    }
}
