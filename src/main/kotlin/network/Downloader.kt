package network

import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.prepareGet
import io.ktor.http.HttpHeaders
import io.ktor.http.Url
import io.ktor.http.contentLength
import io.ktor.http.lastModified
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toKotlinInstant
import kotlinx.datetime.toLocalDateTime
import okio.FileSystem
import okio.Path
import okio.buffer
import schemas.manifest.InstallerManifest
import utils.FileAnalyser
import utils.FileUtils
import utils.Zip
import utils.extension
import utils.findArchitecture
import utils.getFileName
import utils.hashSha256

object Downloader {
    suspend fun download(
        packageIdentifier: String,
        packageVersion: String,
        installerUrl: Url,
        terminal: Terminal,
        fileSystem: FileSystem = FileSystem.SYSTEM
    ): DownloadResult {
        lateinit var fileAnalyser: FileAnalyser
        var zip: Zip? = null
        val downloadedFile = Http.client.downloadFile(installerUrl, packageIdentifier, packageVersion, terminal, fileSystem)
        fileAnalyser = FileAnalyser(downloadedFile.path)
        if (downloadedFile.path.extension.equals(InstallerManifest.InstallerType.ZIP.name, ignoreCase = true)) {
            zip = Zip(zip = downloadedFile.path).also { it.prompt(terminal) }
        }
        try {
            return DownloadResult(
                releaseDate = downloadedFile.lastModified,
                scope = fileAnalyser.scope,
                installerSha256 = downloadedFile.path.hashSha256(),
                installerType = fileAnalyser.installerType,
                upgradeBehaviour = fileAnalyser.upgradeBehaviour,
                architecture = installerUrl.findArchitecture() ?: fileAnalyser.architecture,
                productCode = fileAnalyser.productCode,
                publisherDisplayName = fileAnalyser.publisherDisplayName,
                msix = fileAnalyser.msix,
                msixBundle = fileAnalyser.msixBundle,
                msi = fileAnalyser.msi,
                zip = zip
            )
        } finally {
            fileSystem.delete(downloadedFile.path)
            downloadedFile.removeFileDeletionHook()
        }
    }

    private suspend fun HttpClient.downloadFile(
        url: Url,
        packageIdentifier: String,
        packageVersion: String,
        terminal: Terminal,
        fileSystem: FileSystem = FileSystem.SYSTEM,
        tempDirectory: Path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY,
    ): DownloadedFile {
        val path = FileUtils.createTempFile(packageIdentifier, packageVersion, url, tempDirectory)
        val fileDeletionThread = Thread { fileSystem.delete(path) }
        lateinit var progress: ProgressAnimation
        Runtime.getRuntime().addShutdownHook(fileDeletionThread)
        fileSystem.sink(path).buffer().use { sink ->
            var lastModified: LocalDate? = null
            prepareGet(url).execute { httpResponse ->
                val fileName = httpResponse.headers[HttpHeaders.ContentDisposition]
                    ?.let(::parseFileName)
                    ?: url.getFileName()
                progress = terminal.getProgressBar(fileName).apply(ProgressAnimation::start)
                lastModified = httpResponse
                    .lastModified()
                    ?.toInstant()
                    ?.toKotlinInstant()
                    ?.toLocalDateTime(TimeZone.UTC)
                    ?.date
                val channel: ByteReadChannel = httpResponse.body()
                while (!channel.isClosedForRead) {
                    val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                    while (packet.isNotEmpty) {
                        sink.write(packet.readBytes())
                        fileSystem.metadata(path).size?.let { progress.update(it, httpResponse.contentLength()) }
                    }
                }
                httpResponse.contentLength()?.let { progress.update(it, it) }
            }
            progress.clear()
            return DownloadedFile(path, lastModified, fileDeletionThread, progress)
        }
    }

    private fun parseFileName(contentDisposition: String): String? {
        return Regex("filename=([^;]+)").find(contentDisposition)?.groupValues?.get(1)
    }

    private fun Terminal.getProgressBar(fileName: String?): ProgressAnimation = progressAnimation {
        fileName?.let(::text)
        percentage()
        progressBar()
        completed()
        speed("B/s")
        timeRemaining()
    }
}
