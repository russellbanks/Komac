import Ktor.downloadInstallerFromUrl
import Ktor.getRedirectedUrl
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestChecks
import data.InstallerManifestData
import hashing.Hashing
import hashing.Hashing.hash
import input.Polar
import input.PromptType
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchemaImpl
import schemas.Schemas
import java.io.File

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val installerSchemaImpl: InstallerSchemaImpl = get()

    suspend fun main() {
        with(terminal) {
            packageIdentifierPrompt()
            packageVersionPrompt()
            do {
                installerDownloadPrompt()
                architecturePrompt()
                installerTypePrompt()
                switchPrompt(InstallerSwitch.Silent)
                switchPrompt(InstallerSwitch.SilentWithProgress)
                switchPrompt(InstallerSwitch.Custom)
                installerLocalePrompt()
                productCodePrompt()
                installerScopePrompt()
                upgradeBehaviourPrompt()
                releaseDatePrompt()
                installerManifestData.addInstaller()
                val shouldContinue = shouldLoopPrompt()
            } while (shouldContinue)
            installerManifestData.createInstallerManifest()
        }
    }

    private suspend fun Terminal.packageIdentifierPrompt() {
        do {
            println(brightGreen(Prompts.packageIdentifierInfo))
            installerManifestData.packageIdentifier = prompt(brightWhite(Prompts.packageIdentifier))?.trim()
            installerSchemaImpl.awaitInstallerSchema()
            val (packageIdentifierValid, error) = InstallerManifestChecks.isPackageIdentifierValid(
                installerManifestData.packageIdentifier
            )
            error?.let { println(red(it)) }
            println()
        } while (packageIdentifierValid != Validation.Success)
    }

    private fun Terminal.packageVersionPrompt() {
        do {
            println(brightGreen(Prompts.packageVersionInfo))
            installerManifestData.packageVersion = prompt(brightWhite(Prompts.packageVersion))?.trim()
            val (packageVersionValid, error) = InstallerManifestChecks.isPackageVersionValid(
                installerManifestData.packageVersion
            )
            error?.let { println(red(it)) }
            println()
        } while (packageVersionValid != Validation.Success)
    }

    private suspend fun Terminal.installerDownloadPrompt() {
        do {
            println(brightGreen(Prompts.installerUrlInfo))
            installerManifestData.installerUrl = prompt(brightWhite(PromptType.InstallerUrl.toString()))?.trim()
            val (installerUrlValid, error) = InstallerManifestChecks.isInstallerUrlValid(
                installerManifestData.installerUrl
            )
            error?.let { println(red(it)) }
            println()
        } while (installerUrlValid != Validation.Success)

        val redirectedUrl = getRedirectedUrl(installerManifestData.installerUrl)
        if (
            redirectedUrl != installerManifestData.installerUrl &&
            redirectedUrl?.contains(other = "github", ignoreCase = true) != true
        ) {
            println(brightYellow(Prompts.Redirection.redirectFound))
            println(cyan(Prompts.Redirection.discoveredUrl(redirectedUrl)))
            println((brightGreen(Prompts.Redirection.useDetectedUrl)))
            println(brightWhite(Prompts.Redirection.useOriginalUrl))
            if (prompt(Prompts.enterChoice, default = "Y")?.trim()?.lowercase() != "N".lowercase()) {
                println(brightYellow(Prompts.Redirection.urlChanged))
                val (redirectedUrlValid, error) = InstallerManifestChecks.isInstallerUrlValid(redirectedUrl)
                error?.let { println(it) }
                if (redirectedUrlValid == Validation.Success) {
                    installerManifestData.installerUrl = redirectedUrl
                } else {
                    println()
                    println(brightYellow(Prompts.Redirection.detectedUrlValidationFailed))
                }
                println()
            } else {
                println(brightGreen(Prompts.Redirection.originalUrlRetained(installerManifestData.installerUrl)))
            }
        }

        lateinit var downloadedFile: File
        HttpClient(Java) {
            install(UserAgent) {
                agent = "Microsoft-Delivery-Optimization/10.1"
            }
        }.use { downloadedFile = it.downloadInstallerFromUrl() }
        installerManifestData.installerSha256 = downloadedFile.hash(Hashing.Algorithms.SHA256).uppercase()
        downloadedFile.delete()
        println("Sha256: ${installerManifestData.installerSha256}")
    }

    private fun Terminal.architecturePrompt() {
        do {
            println(brightGreen(Prompts.architectureInfo(installerSchemaImpl)))
            installerManifestData.architecture = prompt(
                brightWhite(PromptType.Architecture.toString())
            )?.trim()?.lowercase()
            val (architectureValid, error) = InstallerManifestChecks.isArchitectureValid(
                installerManifestData.architecture
            )
            error?.let { println(red(it)) }
            println()
        } while (architectureValid != Validation.Success)
    }

    private fun Terminal.installerTypePrompt() {
        do {
            println(brightGreen(Prompts.installerTypeInfo(installerSchemaImpl)))
            installerManifestData.installerType = prompt(brightWhite(Prompts.installerType))?.trim()?.lowercase()
            val (installerTypeValid, error) = InstallerManifestChecks.isInstallerTypeValid(
                installerManifestData.installerType
            )
            error?.let { println(red(it)) }
            println()
        } while (installerTypeValid != Validation.Success)
    }

    private fun Terminal.switchPrompt(installerSwitch: InstallerSwitch) {
        do {
            val infoTextColour = when {
                installerManifestData.installerType == Schemas.InstallerType.exe &&
                    installerSwitch != InstallerSwitch.Custom -> brightGreen
                else -> brightYellow
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

    private fun Terminal.installerLocalePrompt() {
        do {
            println(brightYellow(Prompts.installerLocaleInfo))
            installerManifestData.installerLocale = prompt(brightWhite(PromptType.InstallerLocale.toString()))?.trim()
            val installerLocaleValid = installerSchemaImpl.isInstallerLocaleValid(installerManifestData.installerLocale)
            println()
        } while (installerLocaleValid != Validation.Success)
    }

    private fun Terminal.productCodePrompt() {
        do {
            println(brightYellow(Prompts.productCodeInfo))
            installerManifestData.productCode = prompt(brightWhite(PromptType.ProductCode.toString()))?.trim()
            val productCodeValid = installerSchemaImpl.isProductCodeValid(installerManifestData.productCode)
            println()
        } while (productCodeValid != Validation.Success)
    }

    private fun Terminal.installerScopePrompt() {
        var promptInput: String?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.installerScopeInfo))
                    installerSchemaImpl.installerScopeEnum.forEach { scope ->
                        cell(
                            brightWhite(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${scope.first().titlecase()}] ")
                                    append(scope.replaceFirstChar { it.titlecase() })
                                }
                            )
                        )
                    }
                    cell(
                        brightGreen(
                            buildString {
                                append(" ".repeat(Prompts.optionIndent))
                                append("[${Prompts.noIdea.first().titlecase()}] ")
                                append(Prompts.noIdea)
                            }
                        )
                    )
                }
            )
            promptInput = prompt(brightWhite(Prompts.enterChoice), default = Prompts.noIdea.first().titlecase())?.trim()
            val installerScopeValid = installerSchemaImpl.isInstallerScopeValid(promptInput?.firstOrNull())
            println()
        } while (installerScopeValid != Validation.Success)
        installerManifestData.installerScope = installerSchemaImpl.installerScopeEnum.firstOrNull {
            it.firstOrNull()?.titlecase() == promptInput?.firstOrNull()?.titlecase()
        }
    }

    private fun Terminal.upgradeBehaviourPrompt() {
        var promptInput: String?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.upgradeBehaviourInfo))
                    installerSchemaImpl.upgradeBehaviourEnum.forEach { behaviour ->
                        cell(
                            (
                                if (behaviour.first().titlecase() ==
                                    installerSchemaImpl.upgradeBehaviourEnum.first().first().titlecase()
                                ) {
                                    brightGreen
                                } else {
                                    brightWhite
                                }
                                )(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${behaviour.first().titlecase()}] ")
                                    append(behaviour.replaceFirstChar { it.titlecase() })
                                }
                            )
                        )
                    }
                }
            )
            promptInput = prompt(
                brightWhite(Prompts.enterChoice),
                default = installerSchemaImpl.upgradeBehaviourEnum.first().first().titlecase()
            )?.trim()
            val upgradeBehaviourValid = installerSchemaImpl.isUpgradeBehaviourValid(promptInput?.firstOrNull())
            println()
        } while (upgradeBehaviourValid != Validation.Success)
        installerManifestData.upgradeBehavior = installerSchemaImpl.upgradeBehaviourEnum.firstOrNull {
            it.firstOrNull()?.titlecase() == promptInput?.firstOrNull()?.titlecase()
        }
    }

    private fun Terminal.releaseDatePrompt() {
        do {
            println(brightYellow(Prompts.releaseDateInfo))
            installerManifestData.releaseDate = prompt(brightWhite(PromptType.ReleaseDate.toString()))?.trim()
            val releaseDateValid = installerSchemaImpl.isReleaseDateValid(installerManifestData.releaseDate)
            println()
        } while (releaseDateValid != Validation.Success)
    }

    private fun Terminal.shouldLoopPrompt(): Boolean {
        var promptInput: Char?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.additionalInstallerInfo))
                    cell(
                        brightWhite(
                            "${" ".repeat(Prompts.optionIndent)} [${Polar.Yes.toString().first()}] ${Polar.Yes}"
                        )
                    )
                    cell(
                        brightGreen(
                            "${" ".repeat(Prompts.optionIndent)} [${Polar.No.toString().first()}] ${Polar.No}"
                        )
                    )
                }
            )
            promptInput = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = Polar.No.toString().first().toString()
            )?.trim()?.lowercase()?.firstOrNull()
            println()
        } while (
            promptInput != Polar.Yes.toString().first().lowercaseChar() &&
            promptInput != Polar.No.toString().first().lowercaseChar()
        )
        return promptInput == Polar.Yes.toString().first().lowercaseChar()
    }
}
