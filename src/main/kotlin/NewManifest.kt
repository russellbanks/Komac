import Ktor.downloadInstallerFromUrl
import Ktor.getRedirectedUrl
import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import hashing.Hashing
import hashing.Hashing.hash
import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchemaImpl
import schemas.Schemas

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val installerSchemaImpl: InstallerSchemaImpl = get()

    suspend fun main() {
        with(terminal) {
            packageIdentifierPrompt()
            packageVersionPrompt()
            installerDownloadPrompt()
            architecturePrompt()
            installerTypePrompt()
            switchPrompt(InstallerSwitch.Silent)
            switchPrompt(InstallerSwitch.SilentWithProgress)
            switchPrompt(InstallerSwitch.Custom)
            installerLocalePrompt()
            productCodePrompt()
            installerManifestData.createInstallerManifest()
        }
    }

    private suspend fun Terminal.packageIdentifierPrompt() {
        do {
            println(brightGreen(Prompts.packageIdentifierInfo))
            installerManifestData.packageIdentifier = prompt(brightWhite(Prompts.packageIdentifier))?.trim()
            val packageIdentifierValid = installerSchemaImpl.isPackageIdentifierValid(
                installerManifestData.packageIdentifier
            )
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    private fun Terminal.packageVersionPrompt() {
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            installerManifestData.packageVersion = prompt(brightWhite(Prompts.packageVersion))?.trim()
            val packageVersionValid = installerSchemaImpl.isPackageVersionValid(installerManifestData.packageVersion)
            println()
        } while (packageVersionValid != Validation.Success)
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        val client = HttpClient(Java) {
            install(UserAgent) {
                agent = "Microsoft-Delivery-Optimization/10.1"
            }
            followRedirects = false
        }
        var installerUrlResponse: HttpResponse? = null
        do {
            println(brightGreen(Prompts.installerUrlInfo))
            installerManifestData.installerUrl = prompt(brightWhite(Prompts.installerUrl))?.trim()
            val installerUrlValid = installerSchemaImpl.isInstallerUrlValid(installerManifestData.installerUrl) {
                runCatching { installerUrlResponse = installerManifestData.installerUrl?.let { client.head(it) } }
                installerUrlResponse
            }
            println()
        } while (installerUrlValid != Validation.Success)

        val (redirectedUrl, redirectedUrlResponse) = client.getRedirectedUrl(
            installerManifestData.installerUrl,
            installerUrlResponse
        )
        if (
            redirectedUrl != installerManifestData.installerUrl &&
            redirectedUrl?.contains(other = "github", ignoreCase = true) != true
        ) {
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
                    installerManifestData.installerUrl = redirectedUrl
                } else {
                    println()
                    println(yellow(Prompts.Redirection.detectedUrlValidationFailed))
                }
                println()
            } else {
                println(brightGreen(Prompts.Redirection.originalUrlRetained(installerManifestData.installerUrl)))
            }
        }

        val downloadedFile = client.downloadInstallerFromUrl().also { client.close() }
        installerManifestData.installerSha256 = downloadedFile.hash(Hashing.Algorithms.SHA256).uppercase()

        println("Sha256: ${installerManifestData.installerSha256}")
        downloadedFile.delete()
    }

    private fun Terminal.architecturePrompt() {
        do {
            println(brightGreen(Prompts.architectureInfo(installerSchemaImpl)))
            installerManifestData.architecture = prompt(
                brightWhite(PromptType.Architecture.toString())
            )?.trim()?.lowercase()
            val architectureValid = installerSchemaImpl.isArchitectureValid(installerManifestData.architecture)
            println()
        } while (architectureValid != Validation.Success)
    }

    private fun Terminal.installerTypePrompt() {
        do {
            println(brightGreen(Prompts.installerTypeInfo(installerSchemaImpl)))
            installerManifestData.installerType = prompt(brightWhite(Prompts.installerType))?.trim()?.lowercase()
            val installerTypeValid = installerSchemaImpl.isInstallerTypeValid(installerManifestData.installerType)
            println()
        } while (installerTypeValid != Validation.Success)
    }

    private fun Terminal.switchPrompt(installerSwitch: InstallerSwitch) {
        do {
            val infoTextColour = when {
                installerManifestData.installerType == Schemas.InstallerType.exe &&
                    installerSwitch != InstallerSwitch.Custom -> brightGreen
                else -> yellow
            }
            println(infoTextColour(Prompts.switchInfo(installerManifestData.installerType, installerSwitch)))
            var switchResponse: String? = null
            when (installerSwitch) {
                InstallerSwitch.Silent -> installerManifestData.silentSwitch = prompt(
                    brightWhite(PromptType.SilentSwitch.toString())
                )?.trim().also { switchResponse = it }
                InstallerSwitch.SilentWithProgress -> {
                    installerManifestData.silentWithProgressSwitch = prompt(
                        brightWhite(PromptType.SilentWithProgressSwitch.toString())
                    )?.trim().also { switchResponse = it }
                }
                InstallerSwitch.Custom -> installerManifestData.customSwitch = prompt(
                    brightWhite(PromptType.CustomSwitch.toString())
                )?.trim().also { switchResponse = it }
            }
            val switchValid = installerSchemaImpl.isSwitchValid(
                switch = switchResponse,
                installerSwitch = installerSwitch,
                canBeBlank = installerManifestData.installerType != Schemas.InstallerType.exe ||
                    installerSwitch == InstallerSwitch.Custom
            )
            println()
        } while (switchValid != Validation.Success)
    }

    private fun Terminal.productCodePrompt() {
        do {
            println(yellow(Prompts.productCodeInfo))
            installerManifestData.productCode = prompt(brightWhite(PromptType.ProductCode.toString()))?.trim()
            val productCodeValid = installerSchemaImpl.isProductCodeValid(installerManifestData.productCode)
            println()
        } while (productCodeValid != Validation.Success)
    }

    private fun Terminal.installerLocalePrompt() {
        do {
            println(yellow(Prompts.installerLocaleInfo))
            installerManifestData.installerLocale = prompt(brightWhite(PromptType.InstallerLocale.toString()))?.trim()
            val installerLocaleValid = installerSchemaImpl.isInstallerLocaleValid(installerManifestData.installerLocale)
            println()
        } while (installerLocaleValid != Validation.Success)
    }
}
