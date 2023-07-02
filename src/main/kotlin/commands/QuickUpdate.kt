package commands

import Environment
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.VersionUpdateState
import data.installer.InstallerScope
import data.installer.InstallerType
import data.shared.InstallerUrl
import data.shared.InstallerUrl.downloadInstaller
import data.shared.InstallerUrl.msixBundleDetection
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import github.GitHubDetection
import github.GitHubImpl
import github.GitHubImpl.promptIfPullRequestExists
import github.GitHubUtils
import io.FileWriter
import io.ManifestResultOption
import io.ktor.http.Url
import io.menu.radioMenu
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import network.WebPageScraper
import okio.FileSystem
import okio.Path.Companion.toPath
import org.kohsuke.github.GitHub
import schemas.AdditionalMetadata
import schemas.Schemas
import schemas.installerSorter
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.Manifest
import token.Token
import token.TokenStore
import utils.FileAnalyser
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.UrlsToInstallerMatcher
import utils.findArchitecture
import utils.findScope
import utils.hashSha256
import utils.versionStringComparator

class QuickUpdate : CliktCommand(
    help = """
        Updates a pre-existing manifest with minimal input
        
        Example: komac update --id Package.Identifier --version 1.2.3 --urls https://www.example.com --submit
    """.trimIndent(),
    name = "update"
) {
    private val packageIdentifierParam: String? by option(
        "-i", "--id", "--package-identifier",
        help = "Package identifier. Example: Publisher.Package",
        envvar = "PACKAGE_IDENTIFIER",
    )

    private val packageVersionParam: String? by option(
        "-v", "--version", "--package-version",
        help = "Package version. Example: 1.2.3",
        envvar = "PACKAGE_VERSION"
    )

    private val urls: List<Url>? by option(
        "-u", "--url", "--urls",
        help = "List of new installer URLs. Multiple URLs are delimited by a comma (,)"
    ).convert { Url(it) }.split(",")

    private val manifestOverride: String by option(
        "-mv", "--manifest-version", "--manifest-override",
        help = "Overrides the manifest version.",
        envvar = "MANIFEST_VERSION"
    ).default(Schemas.manifestVersion).check { Regex(Schemas.manifestVersionRegex) matches it }

    private val submit: Boolean by option(
        "-s", "--submit",
        help = "Automatically submits a pull request to remove the manifest"
    ).flag(default = false)

    private val tokenParameter: String? by option(
        "-t", "--token", "--pat", "--personal-access-token",
        help = "GitHub personal access token with the public_repo scope",
        envvar = "GITHUB_TOKEN"
    ).check("The token is invalid or has expired") { GitHub.connectUsingOAuth(it).isCredentialValid }

    private val additionalMetadata by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }

    private var defaultLocale: String? = null
    private var installers = emptyList<InstallerManifest.Installer>()
    private var gitHubDetection: GitHubDetection? = null
    private var pageScraper: WebPageScraper? = null
    private lateinit var packageIdentifier: String
    private lateinit var packageVersion: String
    private lateinit var updateState: VersionUpdateState
    private lateinit var installerUrl: Url
    private lateinit var allVersions: List<String>

    @OptIn(ExperimentalStdlibApi::class)
    override fun run(): Unit = runBlocking {
        tokenParameter?.let { TokenStore.useTokenParameter(it) }
        if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
        if (Environment.isCI) {
            info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
        }
        packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
        if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
        allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)
            ?: throw doesNotExistError(packageIdentifier, isUpdate = true, colors = colors)
        val latestVersion = allVersions.maxWith(versionStringComparator)
        val previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, GitHubImpl.microsoftWinGetPkgs)
        info("Latest version of $packageIdentifier: $latestVersion")
        packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
        currentContext.terminal.promptIfPullRequestExists(identifier = packageIdentifier, version = packageVersion)
        updateState = when (packageVersion) {
            in allVersions -> VersionUpdateState.UpdateVersion
            maxOf(packageVersion, latestVersion, versionStringComparator) -> VersionUpdateState.NewVersion
            else -> VersionUpdateState.AddVersion
        }
        currentContext.terminal.loopThroughInstallers(
            parameterUrls = urls?.toSet(),
            previousManifestData = previousManifestData
        )
        val files = Manifest.createFiles(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale,
            manifestOverride = manifestOverride,
            additionalMetadata = additionalMetadata,
            license = previousManifestData.defaultLocaleManifest!!.license,
            publisher = previousManifestData.defaultLocaleManifest!!.publisher,
            shortDescription = previousManifestData.defaultLocaleManifest!!.shortDescription,
            packageName = previousManifestData.defaultLocaleManifest!!.packageName,
            installers = installers,
            gitHubDetection = gitHubDetection,
            pageScraper = pageScraper,
            previousManifestData = previousManifestData
        )
        for (manifest in files.values.map(Manifest::toString)) {
            formattedManifestLinesSequence(manifest, colors).forEach(::echo)
        }
        if (submit) {
            GitHubImpl.commitAndPullRequest(
                GitHubImpl.getWingetPkgsFork(currentContext.terminal),
                files = files,
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                updateState = updateState,
                previousManifestData = previousManifestData,
                terminal = currentContext.terminal
            ).also { success("Pull request created: ${it.htmlUrl}") }
        } else if (!Environment.isCI) {
            info("What would you like to do with $packageIdentifier $packageVersion?")
            currentContext.terminal.radioMenu<ManifestResultOption> {
                items = ManifestResultOption.entries
                default = ManifestResultOption.PullRequest
            }.prompt().also { manifestResultOption ->
                when (manifestResultOption) {
                    ManifestResultOption.PullRequest -> GitHubImpl.commitAndPullRequest(
                        GitHubImpl.getWingetPkgsFork(currentContext.terminal),
                        files = files,
                        packageIdentifier = packageIdentifier,
                        packageVersion = packageVersion,
                        updateState = updateState,
                        previousManifestData = previousManifestData,
                        terminal = currentContext.terminal
                    ).also { success("Pull request created: ${it.htmlUrl}") }
                    ManifestResultOption.WriteToFiles -> FileWriter.writeFiles(files, currentContext.terminal)
                    else -> return@also
                }
            }
        } else {
            FileWriter.writeFilesToDirectory(
                directory = System.getProperty("user.dir").toPath() / "$packageIdentifier version $packageVersion",
                files = files,
                terminal = currentContext.terminal
            )
        }
    }

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: Set<Url>? = null,
        previousManifestData: PreviousManifestData,
    ) {
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls, previousManifestData)
        } else if (Environment.isCI) {
            throw CliktError(colors.danger("No installers have been provided"), statusCode = 1)
        } else {
            val previousInstallerManifest = previousManifestData.installerManifest.await()!!
            previousInstallerManifest.installers.sortedWith(installerSorter).forEachIndexed { index, installer ->
                info("Installer Entry ${index.inc()}/${previousInstallerManifest.installers.size}")
                listOf(
                    "Architecture" to installer.architecture,
                    InstallerType.name to (installer.installerType ?: previousInstallerManifest.installerType),
                    InstallerScope.name to (installer.scope ?: previousInstallerManifest.scope),
                    "Installer Locale" to (installer.installerLocale ?: previousInstallerManifest.installerLocale)
                ).forEach { (promptType, value) ->
                    value?.let {
                        echo("  $promptType: ${colors.info(it.toString())}")
                    }
                }
                echo()
                installerUrl = prompt(InstallerUrl)
                val installerResult = downloadInstaller(packageIdentifier, packageVersion, installerUrl)
                if (installers.map(InstallerManifest.Installer::installerUrl).contains(installerUrl)) {
                    installers += installers.first { it.installerUrl == installerUrl }
                } else {
                    if (gitHubDetection == null && installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                        gitHubDetection = GitHubDetection(installerUrl)
                    }
                    if (pageScraper == null && !installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                        pageScraper = WebPageScraper(installerUrl)
                    }
                    msixBundleDetection(installerResult.msixBundle)
                    InstallerManifestData.addInstaller(
                        packageVersion = packageVersion,
                        installerUrl = installerUrl,
                        installerSha256 = installerResult.installerSha256,
                        installers = installers,
                        additionalMetadata = additionalMetadata,
                        allVersions = allVersions,
                        architecture = installer.architecture,
                        msix = installerResult.msix,
                        msi = installerResult.msi,
                        msixBundle = installerResult.msixBundle,
                        gitHubDetection = gitHubDetection,
                        zip = installerResult.zip,
                        previousManifestData = previousManifestData
                    ) {
                        installers += it
                    }
                }
            }
        }
    }

    private suspend fun Terminal.loopParameterUrls(
        parameterUrls: Set<Url>,
        previousManifestData: PreviousManifestData
    ) {
        val previousInstallerManifest = previousManifestData.installerManifest.await()!!
        val previousInstallers = previousInstallerManifest.installers
        val previousUrls = previousInstallers.map(InstallerManifest.Installer::installerUrl)
        UrlsToInstallerMatcher.assertUniqueUrlsCount(parameterUrls, previousUrls.toSet(), colors)
        UrlsToInstallerMatcher.assertUrlsValid(parameterUrls, colors)
        val installerResults = mutableListOf<InstallerManifest.Installer>()
        val progressList = parameterUrls.map { url -> getDownloadProgressBar(url).apply(ProgressAnimation::start) }
        gitHubDetection = parameterUrls
            .firstOrNull { it.host.equals(GitHubDetection.gitHubWebsite, ignoreCase = true) }
            ?.let(::GitHubDetection)
        parameterUrls.forEachIndexed { index, url ->
            val downloadedFile = Http.client.downloadFile(url, packageIdentifier, packageVersion, progressList[index])
            val fileAnalyser = FileAnalyser(downloadedFile.path)
            installerResults += try {
                InstallerManifest.Installer(
                    architecture = url.findArchitecture() ?: fileAnalyser.architecture,
                    installerType = fileAnalyser.installerType,
                    scope = url.findScope(),
                    installerSha256 = downloadedFile.path.hashSha256(),
                    installerUrl = url,
                    upgradeBehavior = fileAnalyser.upgradeBehaviour,
                    releaseDate = gitHubDetection?.releaseDate ?: downloadedFile.lastModified
                )
            } finally {
                with(downloadedFile) {
                    FileSystem.SYSTEM.delete(path)
                    removeFileDeletionHook()
                }
            }
        }
        progressList.forEach(ProgressAnimation::clear)
        UrlsToInstallerMatcher.matchInstallers(
            installerResults,
            previousInstallers
                .map {
                    it.copy(
                        installerType = previousInstallerManifest.installerType ?: it.installerType,
                        scope = previousInstallerManifest.scope ?: it.scope
                    )
                }
        ).forEach { (_, newInstaller) ->
            InstallerManifestData.addInstaller(
                packageVersion = packageVersion,
                installerUrl = newInstaller.installerUrl,
                installerSha256 = newInstaller.installerSha256,
                installerType = newInstaller.installerType,
                allVersions = allVersions,
                scope = newInstaller.scope,
                releaseDate = newInstaller.releaseDate,
                upgradeBehavior = newInstaller.upgradeBehavior,
                installers = installers,
                architecture = newInstaller.architecture,
                msix = null,
                msi = null,
                zip = null,
                msixBundle = null,
                gitHubDetection = null,
                previousManifestData = previousManifestData
            ) {
                installers += it
            }
        }
    }
}
