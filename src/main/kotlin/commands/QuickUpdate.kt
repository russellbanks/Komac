package commands
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.multiple
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.installer.Architecture
import data.installer.InstallerScope
import data.installer.InstallerType
import data.shared.Locale
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import input.FileWriter.writeFiles
import input.ManifestResultOption
import input.Prompts
import input.Prompts.pullRequestPrompt
import io.ktor.http.Url
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import ktor.Ktor.decodeHex
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.LocaleManifest
import schemas.manifest.YamlConfig
import java.io.IOException

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
    private val urls: List<Url> by option("--url").convert { Url(it.removeSuffix("/")).decodeHex() }.multiple()
    private val manifestVersion: String? by option()
    private val submit: Boolean by option().flag(default = false)

    override fun run(): Unit = runBlocking {
        manifestVersion?.let { get<SchemasImpl>().manifestOverride = it }
        with(currentContext.terminal) {
            packageIdentifierPrompt(packageIdentifier)
            previousManifestData = get()
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
                warning("${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository.")
                warning("Please use the 'new' command to create a new manifest.")
                return@runBlocking
            }
            launch {
                packageVersionPrompt(packageVersion)
                previousManifestData.remoteInstallerDataJob.join()
                loopThroughInstallers(urls.ifEmpty { null })
                createFiles()
                if (submit) {
                    commit()
                    pullRequest()
                } else {
                    pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                        when (manifestResultOption) {
                            ManifestResultOption.PullRequest -> {
                                commit()
                                pullRequest()
                            }
                            ManifestResultOption.WriteToFiles -> writeFiles(files)
                            else -> echo("Exiting")
                        }
                    }
                }
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(parameterUrls: List<Url>? = null) {
        val remoteInstallerManifest = previousManifestData.remoteInstallerData
        if (parameterUrls?.isNotEmpty() == true) {
            if (remoteInstallerManifest?.installers?.size != parameterUrls.size) {
                throw CliktError(
                    message = "The number of urls provided does not match the number of previous installers",
                    statusCode = 1
                )
            }
            data.shared.Url.areUrlsValid(parameterUrls)
                ?.let { throw CliktError(it) }
                ?: parameterUrls.forEach { url ->
                    installerDownloadPrompt(url)
                    installerManifestData.addInstaller()
                }
        } else {
            remoteInstallerManifest?.installers?.forEachIndexed { index, installer ->
                info("Installer Entry ${index.inc()}/${remoteInstallerManifest.installers.size}")
                listOf(
                    Architecture.const to installer.architecture,
                    InstallerType.const to (installer.installerType ?: remoteInstallerManifest.installerType),
                    InstallerScope.const to (installer.scope ?: remoteInstallerManifest.scope),
                    Locale.installerLocaleConst to
                        (installer.installerLocale ?: remoteInstallerManifest.installerLocale)
                ).forEach { (promptType, value) ->
                    value?.let {
                        println("${" ".repeat(Prompts.optionIndent)} $promptType: ${colors.info(it.toString())}")
                    }
                }
                println()
                installerDownloadPrompt()
                installerManifestData.addInstaller()
            }
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
                manifestVersion = get<SchemasImpl>().localeSchema.properties.manifestVersion.default
            ).let {
                Schemas.buildManifestString(
                    Schema.Locale,
                    YamlConfig.default.encodeToString(LocaleManifest.serializer(), it)
                )
            }
        }.orEmpty()
    }

    private suspend fun Terminal.commit() {
        previousManifestData.remoteVersionDataJob.join()
        previousManifestData.remoteLocaleDataJob.join()
        previousManifestData.remoteDefaultLocaleDataJob.join()
        val repository = githubImpl.getWingetPkgsFork(terminal = this) ?: return
        val ref = githubImpl.createBranchFromDefaultBranch(repository = repository, terminal = this) ?: return
        githubImpl.commitFiles(
            repository = repository,
            branch = ref,
            files = files.map { "${githubImpl.baseGitHubPath}/${it.first}" to it.second }
        )
    }

    private suspend fun Terminal.pullRequest() {
        val ghRepository = githubImpl.getMicrosoftWingetPkgs() ?: return
        try {
            ghRepository.createPullRequest(
                /* title = */ githubImpl.getCommitTitle(),
                /* head = */ "${githubImpl.github.await().myself.login}:${githubImpl.pullRequestBranch?.ref}",
                /* base = */ ghRepository.defaultBranch,
                /* body = */ githubImpl.getPullRequestBody()
            ).also { success("Pull request created: ${it.htmlUrl}") }
        } catch (ioException: IOException) {
            danger(ioException.message ?: "Failed to create pull request")
        }
    }
}
