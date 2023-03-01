package network
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.prepareGet
import io.ktor.http.Url
import io.ktor.http.contentLength
import io.ktor.http.lastModified
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.io.File
import java.time.LocalDateTime
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

object HttpUtils : KoinComponent {
    private val allManifestData: AllManifestData by inject()
    suspend fun HttpClient.downloadFile(url: Url, terminal: Terminal): Pair<File, Thread> = with(allManifestData) {
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                "$packageIdentifier v$packageVersion - $formattedDate",
                ".${url.getExtension()}"
            )
        }
        val fileDeletionThread = Thread { file.delete() }
        Runtime.getRuntime().addShutdownHook(fileDeletionThread)
        with(terminal) {
            getDownloadProgressBar(url, this).run {
                start()
                prepareGet(url).execute { httpResponse ->
                    httpResponse.lastModified()?.let {
                        releaseDate = it.toInstant().atOffset(ZoneOffset.UTC).toLocalDate()
                    }
                    val channel: ByteReadChannel = httpResponse.body()
                    while (!channel.isClosedForRead) {
                        val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                        while (packet.isNotEmpty) {
                            file.appendBytes(packet.readBytes())
                            update(file.length(), httpResponse.contentLength())
                        }
                    }
                }
                clear()
            }
        }
        return file to fileDeletionThread
    }

    fun getDownloadProgressBar(url: Url, terminal: Terminal): ProgressAnimation {
        return terminal.progressAnimation {
            url.getFileName()?.let { text(it) }
            percentage()
            progressBar()
            completed()
            speed("B/s")
            timeRemaining()
        }
    }

    fun getDirectoryPath(packageIdentifier: String): String {
        return buildString {
            append("manifests/")
            append(packageIdentifier.first().lowercase())
            packageIdentifier.split(".").forEach { append("/$it") }
        }
    }
}
