package ktor
import com.github.ajalt.mordant.animation.progressAnimation
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
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.commons.io.FilenameUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.TerminalInstance
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object Ktor : KoinComponent {
    suspend fun HttpClient.downloadInstallerFromUrl(): File {
        val terminalInstance: TerminalInstance by inject()
        val sharedManifestData: SharedManifestData by inject()
        val installerManifestData: InstallerManifestData by inject()
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                "${sharedManifestData.packageIdentifier} v${sharedManifestData.packageVersion} - $formattedDate",
                ".${getURLExtension(Url(installerManifestData.installerUrl))}"
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
                prepareGet(installerManifestData.installerUrl).execute { httpResponse ->
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
        return file
    }

    fun getDirectoryPath(packageIdentifier: String): String {
        return buildString {
            append("manifests/")
            append(packageIdentifier.first().lowercase())
            packageIdentifier.split(".").forEach { append("/$it") }
        }
    }

    private fun getURLExtension(url: Url): String {
        return url.fullPath
            .substringAfterLast(".")
            .split(Regex("[^A-Za-z0-9]"))
            .firstOrNull()
            .also { get<SharedManifestData>().fileExtension = it } ?: "winget-tmp"
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

    suspend fun getRedirectedUrl(installerUrl: String): String? {
        val noRedirectClient = get<Clients>().httpClient.config { followRedirects = false }
        var redirectedInstallerUrl: String? = installerUrl
        var response: HttpResponse? = noRedirectClient.head(installerUrl)

        var status: HttpStatusCode? = response?.status
        var location: String? = response?.headers?.get("Location")
        while (
            status?.isRedirect() == true &&
            response?.headers?.contains(HttpHeaders.Location) == true &&
            location != null
        ) {
            redirectedInstallerUrl = location
            response = noRedirectClient.head(redirectedInstallerUrl)
            status = response.status
            location = response.headers["Location"]
        }
        noRedirectClient.close()
        return redirectedInstallerUrl
    }

    const val userAgent = "Microsoft-Delivery-Optimization/10.1"
}
