import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.optional
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.YamlConfig
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import input.ManifestResultOption
import input.PromptType
import input.Prompts
import input.Prompts.pullRequestPrompt
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.LocaleManifest
import schemas.SchemasImpl
import schemas.TerminalInstance
import schemas.VersionManifest
import java.io.File

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val urls: List<String>? by argument("--urls").multiple().optional()

    private val installerManifestData: InstallerManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var previousManifestData: PreviousManifestData
    private lateinit var files: List<Pair<String, String?>>
    private val githubImpl: GitHubImpl by inject()

    override fun run(): Unit = runBlocking {
        val githubImpl: GitHubImpl by inject()
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt()
            previousManifestData = get()
            if (sharedManifestData.isNewPackage) {
                println(
                    verticalLayout {
                        cell(
                            brightYellow(
                                "${sharedManifestData.packageIdentifier} is not in the " +
                                    "${GitHubImpl.wingetpkgs} repository."
                            )
                        )
                        cell(brightYellow("Please use the 'new' command to create a new manifest."))
                    }
                )
                return@runBlocking
            }
            launch {
                packageVersionPrompt()
                previousManifestData.remoteInstallerDataJob.join()
                loopThroughInstallers()
                createFiles()
                pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> commitAndPullRequest()
                        ManifestResultOption.WriteToFiles -> writeFiles()
                        else -> println(brightWhite("Exiting"))
                    }
                }
            }
        }
    }

    private fun Terminal.writeFiles() {
        do {
            println()
            println(brightYellow("Enter a directory to write the files to:"))
            val directory = prompt(brightWhite("Directory"))?.let { File(it) }
            if (directory?.isDirectory == true) {
                writeFilesToDirectory(directory)
            } else {
                println("The directory entered is not a valid directory")
            }
        } while (directory?.isDirectory != true)
    }

    private fun writeFilesToDirectory(directory: File) {
        files.forEach { file ->
            file.second?.let { manifestText ->
                writeFileToDirectory(directory, file.first, manifestText)
            }
        }
    }

    private fun writeFileToDirectory(directory: File, fileName: String, fileText: String) {
        File(directory, fileName).apply {
            if (canWrite()) {
                writeText(fileText)
                if (exists()) {
                    println(brightGreen("Successfully written $name to ${directory.path}"))
                } else {
                    println(red("Failed to write $name"))
                }
            } else {
                println(red("Cannot write to $name"))
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers() {
        val remoteInstallers = previousManifestData.remoteInstallerData?.installers
        do {
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
            (previousManifestData.remoteInstallerData?.installers?.size ?: 0) <
            installerManifestData.installers.size
        )
    }

    private fun createFiles() {
        sharedManifestData.defaultLocale = previousManifestData.remoteVersionData!!.defaultLocale
        files = listOf(
            githubImpl.installerManifestName to previousManifestData.remoteInstallerData?.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                installers = installerManifestData.installers,
                manifestVersion = "1.4.0"
            )?.let {
                githubImpl.buildManifestString(get<SchemasImpl>().installerSchema.id) {
                    appendLine(
                        YamlConfig.defaultWithLocalDataSerializer.encodeToString(InstallerManifest.serializer(), it)
                    )
                }
            },
            githubImpl.defaultLocaleManifestName to previousManifestData.remoteDefaultLocaleData?.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                manifestVersion = "1.4.0"
            )?.let {
                githubImpl.buildManifestString(get<SchemasImpl>().defaultLocaleSchema.id) {
                    appendLine(YamlConfig.default.encodeToString(DefaultLocaleManifest.serializer(), it))
                }
            },
            githubImpl.versionManifestName to previousManifestData.remoteVersionData?.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                manifestVersion = "1.4.0"
            )?.let {
                githubImpl.buildManifestString(get<SchemasImpl>().versionSchema.id) {
                    appendLine(YamlConfig.default.encodeToString(VersionManifest.serializer(), it))
                }
            }
        ) + previousManifestData.remoteLocaleData?.map { localeManifest ->
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                manifestVersion = "1.4.0"
            ).let {
                githubImpl.buildManifestString(get<SchemasImpl>().localeSchema.id) {
                    appendLine(YamlConfig.default.encodeToString(LocaleManifest.serializer(), it))
                }
            }
        }.orEmpty()
    }

    private suspend fun commitAndPullRequest() {
        previousManifestData.remoteVersionDataJob.join()
        previousManifestData.remoteLocaleDataJob.join()
        previousManifestData.remoteDefaultLocaleDataJob.join()
        val repository = githubImpl.getWingetPkgsFork() ?: return
        val ref = githubImpl.createBranchFromDefaultBranch(repository) ?: return
        githubImpl.commitFiles(
            repository = repository,
            branch = ref,
            files = files.map { "${githubImpl.baseGitHubPath}/${it.first}" to it.second }
        )
    }
}
