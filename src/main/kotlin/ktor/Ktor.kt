package ktor
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.SharedManifestData
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.head
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import io.ktor.http.contentLength
import io.ktor.http.fullPath
import io.ktor.http.lastModified
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import java.io.File
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

object Ktor : KoinComponent {
    suspend fun HttpClient.downloadInstallerFromUrl(terminal: Terminal): Pair<File, Thread> {
        val sharedManifestData: SharedManifestData by inject()
        val installerManifestData: InstallerManifestData by inject()
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                "${sharedManifestData.packageIdentifier} v${sharedManifestData.packageVersion} - $formattedDate",
                ".${getURLExtension(installerManifestData.installerUrl)}"
            )
        }
        val fileDeletionThread = Thread { file.delete() }
        Runtime.getRuntime().addShutdownHook(fileDeletionThread)

        with(terminal) {
            progressAnimation {
                getFileName(installerManifestData.installerUrl)?.let { text(it) }
                percentage()
                progressBar()
                completed()
                speed("B/s")
                timeRemaining()
            }.run {
                start()
                prepareGet(installerManifestData.installerUrl).execute { httpResponse ->
                    httpResponse.lastModified()?.let {
                        installerManifestData.releaseDate = LocalDate.ofInstant(it.toInstant(), ZoneId.systemDefault())
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
                stop()
                clear()
            }
        }
        return file to fileDeletionThread
    }

    fun getDirectoryPath(packageIdentifier: String): String {
        return buildString {
            append("manifests/")
            append(packageIdentifier.first().lowercase())
            packageIdentifier.split(".").forEach { append("/$it") }
        }
    }

    private fun getFileName(url: Url): String? {
        return url.pathSegments.findLast { it.endsWith(getURLExtension(url)) }
    }

    fun getURLExtension(url: Url): String {
        return url.fullPath.substringAfterLast(".").split(Regex("[^A-Za-z0-9]")).firstOrNull() ?: "winget-tmp"
    }

    fun detectArchitectureFromUrl(url: Url): InstallerManifest.Installer.Architecture? {
        var archInUrl = Regex("(x86_64|i[3-6]86|x\\d+|arm(?:64)?|aarch(?:64)?)")
            .find(url.fullPath.lowercase())?.groupValues?.last()
        when (archInUrl) {
            "aarch" -> archInUrl = InstallerManifest.Installer.Architecture.ARM.toString()
            "aarch64" -> archInUrl = InstallerManifest.Installer.Architecture.ARM64.toString()
            "x86_64" -> archInUrl = InstallerManifest.Installer.Architecture.X64.toString()
            "i386", "i486", "i586", "i686" -> archInUrl = InstallerManifest.Installer.Architecture.X86.toString()
        }
        return try {
            InstallerManifest.Installer.Architecture.valueOf(archInUrl?.uppercase() ?: "")
        } catch (_: IllegalArgumentException) {
            null
        }
    }

    fun HttpStatusCode.isRedirect(): Boolean {
        return value in HttpStatusCode.MovedPermanently.value..HttpStatusCode.PermanentRedirect.value
    }

    suspend fun getRedirectedUrl(installerUrl: Url): Url {
        val noRedirectClient = get<Clients>().httpClient.config { followRedirects = false }
        var redirectedInstallerUrl: Url = installerUrl
        var response: HttpResponse? = noRedirectClient.head(installerUrl)

        var status: HttpStatusCode? = response?.status
        var location: String? = response?.headers?.get(HttpHeaders.Location)
        while (
            status?.isRedirect() == true &&
            response?.headers?.contains(HttpHeaders.Location) == true &&
            location != null
        ) {
            redirectedInstallerUrl = Url(location)
            response = noRedirectClient.head(redirectedInstallerUrl)
            status = response.status
            location = response.headers[HttpHeaders.Location]
        }
        noRedirectClient.close()
        return redirectedInstallerUrl
    }

    const val userAgent = "Microsoft-Delivery-Optimization/10.1"
}
