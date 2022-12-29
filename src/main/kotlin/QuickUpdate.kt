import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.optional
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.GitHubImpl
import data.InstallerManifestData
import data.SharedManifestData
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import input.PromptType
import input.Prompts
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.ManifestBuilder
import schemas.ManifestBuilder.writeManifestsToFiles
import schemas.TerminalInstance
import kotlin.system.exitProcess

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val urls: List<String>? by argument("--urls").multiple().optional()

    private val installerManifestData: InstallerManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt()
            exitIfNotInWingetPkgs()
            launch { sharedManifestData.getPreviousManifestData() }
            launch {
                packageVersionPrompt()
                do {
                    val remoteInstallers = sharedManifestData.remoteInstallerData?.installers
                    remoteInstallers?.forEachIndexed { index, installer ->
                        println(
                            verticalLayout {
                                cell(brightGreen("Installer Entry ${index.inc()}/${remoteInstallers.size}"))
                                listOf(
                                    PromptType.Architecture to installer.architecture,
                                    PromptType.InstallerType to installer.installerType,
                                    PromptType.Scope to installer.scope,
                                    PromptType.InstallerLocale to installer.installerLocale
                                ).forEach { (promptType, value) ->
                                    value?.let {
                                        cell(brightYellow("${" ".repeat(Prompts.optionIndent)} $promptType: $it"))
                                    }
                                }
                                cell("")
                            }
                        )
                        installerDownloadPrompt()
                        installerManifestData.installers += installer.copy(
                            installerUrl = installerManifestData.installerUrl,
                            installerSha256 = installerManifestData.installerSha256,
                            productCode = installerManifestData.productCode,
                        )
                    }
                } while (
                    (sharedManifestData.remoteInstallerData?.installers?.size ?: 0) <
                    installerManifestData.installers.size
                )
                sharedManifestData.defaultLocale = sharedManifestData.remoteVersionData!!.defaultLocale
                val (komacTemp, versionDirectory) = ManifestBuilder.createTempDirectories()
                sharedManifestData.remoteInstallerDataJob?.join()
                sharedManifestData.remoteDefaultLocaleDataJob?.join()
                sharedManifestData.remoteLocaleDataJob?.join()
                sharedManifestData.remoteVersionDataJob?.join()
                versionDirectory.writeManifestsToFiles(
                    installerManifest = sharedManifestData.remoteInstallerData?.copy(
                        packageIdentifier = sharedManifestData.packageIdentifier,
                        packageVersion = sharedManifestData.packageVersion,
                        installers = installerManifestData.installers,
                        manifestVersion = "1.4.0"
                    ),
                    defaultLocaleManifest = sharedManifestData.remoteDefaultLocaleData?.copy(
                        packageIdentifier = sharedManifestData.packageIdentifier,
                        packageVersion = sharedManifestData.packageVersion,
                        manifestVersion = "1.4.0"
                    ),
                    localeManifests = sharedManifestData.remoteLocaleData,
                    versionManifest = sharedManifestData.remoteVersionData?.copy(
                        packageIdentifier = sharedManifestData.packageIdentifier,
                        packageVersion = sharedManifestData.packageVersion,
                        manifestVersion = "1.4.0"
                    )
                )
                try {
                    val githubImpl = get<GitHubImpl>()
                    val repository = githubImpl.getWingetPkgsFork(this@with)
                    val ref = githubImpl.createBranch(repository)
                    githubImpl.commitFiles(repository = repository, branch = ref, versionDirectory = versionDirectory)
                } finally {
                    komacTemp.toFile().deleteRecursively()
                }
            }
        }
    }

    private fun Terminal.exitIfNotInWingetPkgs() {
        if (sharedManifestData.isNewPackage) {
            println(
                verticalLayout {
                    cell(
                        brightYellow(
                            "${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository."
                        )
                    )
                    cell(brightYellow("Please use the 'new' command to create a new manifest."))
                }
            )
            exitProcess(0)
        }
    }
}
