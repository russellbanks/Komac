package network

import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.prepareGet
import io.ktor.http.Url
import io.ktor.http.contentLength
import io.ktor.http.lastModified
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import utils.FileUtils
import utils.getFileName
import java.io.File
import java.time.LocalDate
import java.time.ZoneOffset

object HttpUtils {
    suspend fun HttpClient.downloadFile(
        url: Url,
        packageIdentifier: String,
        packageVersion: String,
        progress: ProgressAnimation? = null
    ): DownloadedFile {
        val file = FileUtils.createTempFile(identifier = packageIdentifier, version = packageVersion, url = url)
        val fileDeletionThread = Thread { file.delete() }
        var lastModified: LocalDate? = null
        prepareGet(url).execute { httpResponse ->
            lastModified = httpResponse.lastModified()?.toInstant()?.atZone(ZoneOffset.UTC)?.toLocalDate()
            val channel: ByteReadChannel = httpResponse.body()
            while (!channel.isClosedForRead) {
                val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                while (packet.isNotEmpty) {
                    file.appendBytes(packet.readBytes())
                    progress?.update(file.length(), httpResponse.contentLength())
                }
            }
        }
        return DownloadedFile(file, lastModified, fileDeletionThread)
    }

    data class DownloadedFile(val file: File, val lastModified: LocalDate?, val fileDeletionThread: Thread)

    fun Terminal.getDownloadProgressBar(url: Url): ProgressAnimation {
        return progressAnimation {
            url.getFileName()?.let(::text)
            percentage()
            progressBar()
            completed()
            speed("B/s")
            timeRemaining()
        }
    }
}
