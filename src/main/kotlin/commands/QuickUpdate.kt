package commands
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.multiple
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.YamlConfig
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url
import data.shared.Url.installerDownloadPrompt
import input.FileWriter.writeFiles
import input.ManifestResultOption
import input.PromptType
import input.Prompts
import input.Prompts.pullRequestPrompt
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.TerminalInstance
import schemas.manifest.LocaleManifest
import kotlin.system.exitProcess

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var previousManifestData: PreviousManifestData
    private lateinit var files: List<Pair<String, String?>>
    private val githubImpl: GitHubImpl by inject()
    private val packageIdentifier: String? by option()
    private val packageVersion: String? by option()
    private val urls: List<String> by option("--url").multiple()

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt(packageIdentifier)
            previousManifestData = get()
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
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
                packageVersionPrompt(packageVersion)
                previousManifestData.remoteInstallerDataJob.join()
                loopThroughInstallers(urls)
                createFiles()
                pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> commitAndPullRequest()
                        ManifestResultOption.WriteToFiles -> writeFiles(files)
                        else -> println(brightWhite("Exiting"))
                    }
                }
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(parameterUrls: List<String> = emptyList()) {
        val remoteInstallers = previousManifestData.remoteInstallerData?.installers
        if (parameterUrls.isNotEmpty()) {
            if (remoteInstallers?.size != parameterUrls.size) {
                println(brightRed("The number of urls provided does not match the number of previous installers"))
                exitProcess(0)
            }
            val urlError = Url.areUrlsValid(parameterUrls)
            if (urlError == null) {
                parameterUrls.forEach { url ->
                    installerDownloadPrompt(url)
                    installerManifestData.addInstaller()
                }
            } else {
                println(brightRed(urlError))
                exitProcess(0)
            }
        } else {
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
                    installerManifestData.addInstaller()
                }
            } while (
                (previousManifestData.remoteInstallerData?.installers?.size ?: 0) <
                installerManifestData.installers.size
            )
        }
    }

    private suspend fun createFiles() {
        sharedManifestData.defaultLocale = previousManifestData.remoteVersionData!!.defaultLocale
        files = listOf(
            githubImpl.installerManifestName to installerManifestData.createInstallerManifest(),
            githubImpl.defaultLocaleManifestName to defaultLocaleManifestData.createDefaultLocaleManifest(),
            githubImpl.versionManifestName to versionManifestData.createVersionManifest()
        ) + previousManifestData.remoteLocaleData?.map { localeManifest ->
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                manifestVersion = "1.4.0"
            ).let {
                Schemas.buildManifestString(
                    Schema.Locale,
                    YamlConfig.default.encodeToString(LocaleManifest.serializer(), it)
                )
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
