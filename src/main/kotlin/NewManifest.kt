import Ktor.isRedirect
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import hashing.Hashing
import hashing.Hashing.hash
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.head
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpHeaders
import io.ktor.http.contentLength
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.commons.io.FilenameUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.InstallerManifest
import schemas.InstallerSchemaImpl
import schemas.Schemas
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private var packageVersion: String? = null
    private var installerUrl: String? = null
    private var packageIdentifier: String? = null
    private var installerSha256: String? = null
    private var architecture: String? = null
    private var installerType: String? = null
    private var silentSwitch: String? = null
    private var silentWithProgressSwitch: String? = null
    private var customSwitch: String? = null
    private val installerSchemaImpl: InstallerSchemaImpl = get()

    private val client = HttpClient(Java) {
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
            downloadInstallerFromUrl(installerUrl)
            architecturePrompt()
            installerTypePrompt()
            switchPrompt(InstallerSwitch.Silent)
            switchPrompt(InstallerSwitch.SilentWithProgress)
            switchPrompt(InstallerSwitch.Custom)
            InstallerManifest(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                installers = listOf(
                    InstallerManifest.Installer(
                        architecture = architecture,
                        installerType = installerType,
                        installerUrl = installerUrl,
                        installerSha256 = installerSha256,
                        installerSwitches = InstallerManifest.Installer.InstallerSwitches(
                            silent = silentSwitch?.ifBlank { null },
                            silentWithProgress = silentWithProgressSwitch?.ifBlank { null },
                            custom = customSwitch?.ifBlank { null }
                        ),
                    )
                ),
                manifestVersion = Schemas.manifestVersion
            ).also {
                Yaml(
                    configuration = YamlConfiguration(
                        encodeDefaults = false,
                        singleLineStringStyle = SingleLineStringStyle.Plain
                    )
                ).run {
                    buildString {
                        appendLine(Schemas.Comments.createdBy)
                        appendLine(Schemas.Comments.installerLanguageServer)
                        appendLine()
                        appendLine(encodeToString(InstallerManifest.serializer(), it))
                    }.let(this@with::print)
                }
            }
        }
    }

    private suspend fun Terminal.packageIdentifierPrompt() {
        do {
            println(brightGreen(Prompts.packageIdentifierInfo))
            packageIdentifier = prompt(brightWhite(Prompts.packageIdentifier))?.trim()
            val packageIdentifierValid = installerSchemaImpl.isPackageIdentifierValid(packageIdentifier)
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    private fun Terminal.packageVersionPrompt() {
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            packageVersion = prompt(brightWhite(Prompts.packageVersion))?.trim()
            val packageVersionValid = installerSchemaImpl.isPackageVersionValid(packageVersion)
            println()
        } while (packageVersionValid != Validation.Success)
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        var installerUrlResponse: HttpResponse? = null
        do {
            println(brightGreen(Prompts.installerUrlInfo))
            installerUrl = prompt(brightWhite(Prompts.installerUrl))?.trim()
            val installerUrlValid = installerSchemaImpl.isInstallerUrlValid(installerUrl) {
                runCatching { installerUrlResponse = installerUrl?.let { client.head(it) } }
                installerUrlResponse
            }
            println()
        } while (installerUrlValid != Validation.Success)

        val (redirectedUrl, redirectedUrlResponse) = client.getRedirectedUrl(installerUrl, installerUrlResponse)
        if (redirectedUrl != installerUrl && redirectedUrl?.contains(other = "github", ignoreCase = true) != true) {
            println(yellow(Prompts.Redirection.redirectFound))
            println(blue(Prompts.Redirection.discoveredUrl(redirectedUrl)))
            println((brightGreen(Prompts.Redirection.useDetectedUrl)))
            println(brightWhite(Prompts.Redirection.useOriginalUrl))
            if (prompt(Prompts.Redirection.enterChoice, default = "Y")?.trim()?.lowercase() != "N".lowercase()) {
                println(yellow(Prompts.Redirection.urlChanged))
                val redirectedUrlValid = installerSchemaImpl.isInstallerUrlValid(redirectedUrl) {
                    redirectedUrlResponse
                }
                if (redirectedUrlValid == Validation.Success) {
                    installerUrl = redirectedUrl
                } else {
                    println()
                    println(yellow(Prompts.Redirection.detectedUrlValidationFailed))
                }
                println()
            } else {
                println(brightGreen(Prompts.Redirection.originalUrlRetained(installerUrl)))
            }
        }
    }

    private suspend fun Terminal.downloadInstallerFromUrl(installerUrl: String?) {
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        val file = withContext(Dispatchers.IO) {
            File.createTempFile(
                /* prefix = */ "$packageIdentifier v$packageVersion - $formattedDate",
                /* suffix = */ ".${getURLExtension(installerUrl)}"
            )
        }

        progressAnimation {
            text(FilenameUtils.getName(installerUrl))
            percentage()
            progressBar()
            completed()
            speed("B/s")
            timeRemaining()
        }.run {
            start()
            client.config { followRedirects = true }.use { client ->
                client.prepareGet(installerUrl as String).execute { httpResponse ->
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
        client.close()
        installerSha256 = file.hash(Hashing.Algorithms.SHA256).uppercase()

        println("Sha256: $installerSha256")
        file.delete()
    }

    private fun Terminal.architecturePrompt() {
        do {
            println(brightGreen(Prompts.architectureInfo(installerSchemaImpl)))
            architecture = prompt(brightWhite(Prompts.architecture))?.trim()?.lowercase()
            val architectureValid = installerSchemaImpl.isArchitectureValid(architecture)
            println()
        } while (architectureValid != Validation.Success)
    }

    private fun Terminal.installerTypePrompt() {
        do {
            println(brightGreen(Prompts.installerTypeInfo(installerSchemaImpl)))
            installerType = prompt(brightWhite(Prompts.installerType))?.trim()?.lowercase()
            val installerTypeValid = installerSchemaImpl.isInstallerTypeValid(installerType)
            println()
        } while (installerTypeValid != Validation.Success)
    }

    private fun Terminal.switchPrompt(installerSwitch: InstallerSwitch) {
        do {
            val infoTextColour = when {
                installerType == Schemas.InstallerType.exe && installerSwitch != InstallerSwitch.Custom -> brightGreen
                else -> yellow
            }
            println(infoTextColour(Prompts.switchInfo(installerType, installerSwitch)))
            var switchResponse: String? = null
            when (installerSwitch) {
                InstallerSwitch.Silent -> silentSwitch = prompt(
                    brightWhite(PromptType.SilentSwitch.toString())
                )?.trim().also { switchResponse = it }
                InstallerSwitch.SilentWithProgress -> {
                    silentWithProgressSwitch = prompt(
                        brightWhite(PromptType.SilentWithProgressSwitch.toString())
                    )?.trim().also { switchResponse = it }
                }
                InstallerSwitch.Custom -> customSwitch = prompt(
                    brightWhite(PromptType.CustomSwitch.toString())
                )?.trim().also { switchResponse = it }
            }
            val switchValid = installerSchemaImpl.isSwitchValid(
                switch = switchResponse,
                installerSwitch = installerSwitch,
                canBeBlank = installerType != Schemas.InstallerType.exe || installerSwitch == InstallerSwitch.Custom
            )
            println()
        } while (switchValid != Validation.Success)
    }

    private suspend fun HttpClient.getRedirectedUrl(
        installerUrl: String?,
        httpResponse: HttpResponse?
    ): Pair<String?, HttpResponse?> {
        var redirectedInstallerUrl: String? = installerUrl
        var newResponse: HttpResponse? = httpResponse

        var status = httpResponse?.status
        var location = httpResponse?.headers?.get("Location")
        while (
            status?.isRedirect() == true &&
            httpResponse?.headers?.contains(HttpHeaders.Location) == true &&
            location != null
        ) {
            redirectedInstallerUrl = location
            newResponse = head(redirectedInstallerUrl)
            status = newResponse.status
            location = newResponse.headers["Location"]
        }
        return redirectedInstallerUrl to newResponse
    }

    private fun getURLExtension(url: String?): String {
        var urlExtension: String? = FilenameUtils.getExtension(url)
        if (urlExtension.isNullOrBlank()) urlExtension = "winget-tmp"
        return urlExtension
    }
}
