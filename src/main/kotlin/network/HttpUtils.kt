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
import okio.FileSystem
import okio.Path
import okio.buffer
import utils.FileUtils
import utils.getFileName
import java.time.LocalDate
import java.time.ZoneOffset

object HttpUtils {
    suspend fun HttpClient.downloadFile(
        url: Url,
        packageIdentifier: String,
        packageVersion: String,
        progress: ProgressAnimation? = null,
        fileSystem: FileSystem,
        tempDirectory: Path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY
    ): DownloadedFile {
        val path = FileUtils.createTempFile(packageIdentifier, packageVersion, url, tempDirectory)
        val fileDeletionThread = Thread { fileSystem.delete(path) }
        fileSystem.sink(path).buffer().use { sink ->
            var lastModified: LocalDate? = null
            prepareGet(url).execute { httpResponse ->
                lastModified = httpResponse.lastModified()?.toInstant()?.atZone(ZoneOffset.UTC)?.toLocalDate()
                val channel: ByteReadChannel = httpResponse.body()
                while (!channel.isClosedForRead) {
                    val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                    while (packet.isNotEmpty) {
                        sink.write(packet.readBytes())
                        fileSystem.metadata(path).size?.let { progress?.update(it, httpResponse.contentLength()) }
                    }
                }
            }
            return DownloadedFile(path, lastModified, fileDeletionThread)
        }
    }

    data class DownloadedFile(val path: Path, val lastModified: LocalDate?, val fileDeletionHook: Thread) {
        fun removeFileDeletionHook() = Runtime.getRuntime().removeShutdownHook(fileDeletionHook)
    }

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
