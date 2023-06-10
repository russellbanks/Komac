package commands

import Environment
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import github.GitHubImpl
import data.InstallerManifestData
import data.ManifestData
import data.PreviousManifestData
import data.VersionManifestData
import data.installer.InstallerScope
import data.installer.InstallerType
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
import utils.UrlsToInstallerMatcher
import github.GitHubDetection
import utils.hashSha256
import utils.versionStringComparator
import io.FileWriter
import io.ManifestResultOption
import io.menu.radioMenu
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import okio.FileSystem
import okio.Path.Companion.toPath
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import schemas.AdditionalMetadata
import schemas.Schemas
import schemas.installerSorter
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import token.Token
import token.TokenStore
import utils.FileAnalyser
import github.GitHubUtils
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.findArchitecture
import utils.findScope

class QuickUpdate : CliktCommand(
    help = """
        Updates a pre-existing manifest with minimal io
        
        Example: komac update --id Package.Identifier --version 1.2.3 --urls https://www.example.com --submit
    """.trimIndent(),
    name = "update"
) {
    private val packageIdentifierParam: String? by option(
        "-i", "--id", "--package-identifier",
        help = "Package identifier. Example: Publisher.Package"
    )

    private val packageVersionParam: String? by option(
        "-v", "--version", "--package-version",
        help = "Package version. Example: 1.2.3"
    )

    private val urls: List<Url>? by option(
        "-u", "--url", "--urls",
        help = "List of new installer URLs. Multiple URLs are delimited by a comma (,)"
    ).convert { Url(it) }.split(",")

    private val manifestOverride: String? by option(
        "-mv", "--manifest-version", "--manifest-override",
        help = "Overrides the manifest version.",
        envvar = "MANIFEST_VERSION"
    ).check { Regex(Schemas.manifestVersionRegex) matches it }

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

    private lateinit var microsoftWingetPkgs: GHRepository

    @OptIn(ExperimentalStdlibApi::class)
    override fun run(): Unit = runBlocking {
        tokenParameter?.let { TokenStore.useTokenParameter(it) }
        if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
        if (Environment.isCI) {
            info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
        }
        ManifestData.packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
        if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
        microsoftWingetPkgs = GitHubImpl.microsoftWinGetPkgs
        ManifestData.allVersions = GitHubUtils.getAllVersions(microsoftWingetPkgs, ManifestData.packageIdentifier)
            ?.also { info("Found ${ManifestData.packageIdentifier} in the winget-pkgs repository") }
            ?: throw doesNotExistError(ManifestData.packageIdentifier, isUpdate = true, colors = colors)
        val latestVersion = (ManifestData.allVersions as List<String>).maxWith(versionStringComparator)
        info("Found latest version: $latestVersion")
        PreviousManifestData.init(ManifestData.packageIdentifier, latestVersion, microsoftWingetPkgs)
        ManifestData.packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
        GitHubImpl.promptIfPullRequestExists(
            identifier = ManifestData.packageIdentifier,
            version = ManifestData.packageVersion,
            terminal = currentContext.terminal
        )
        ManifestData.updateState = getUpdateState(
            ManifestData.packageIdentifier,
            ManifestData.packageVersion, latestVersion)
        currentContext.terminal.loopThroughInstallers(parameterUrls = urls?.toSet())
        val files = createFiles(ManifestData.packageIdentifier, ManifestData.packageVersion, ManifestData.defaultLocale)
        for (manifest in files.values) {
            formattedManifestLinesSequence(manifest, colors).forEach(::echo)
        }
        if (submit) {
            GitHubImpl.commitAndPullRequest(
                GitHubImpl.getWingetPkgsFork(currentContext.terminal),
                files = files,
                packageIdentifier = ManifestData.packageIdentifier,
                packageVersion = ManifestData.packageVersion,
                updateState = ManifestData.updateState
            ).also { success("Pull request created: ${it.htmlUrl}") }
        } else if (!Environment.isCI) {
            info("What would you like to do with ${ManifestData.packageIdentifier} ${ManifestData.packageVersion}?")
            currentContext.terminal.radioMenu<ManifestResultOption> {
                items = ManifestResultOption.entries
                default = ManifestResultOption.PullRequest
            }.prompt().also { manifestResultOption ->
                when (manifestResultOption) {
                    ManifestResultOption.PullRequest -> GitHubImpl.commitAndPullRequest(
                        GitHubImpl.getWingetPkgsFork(currentContext.terminal),
                        files = files,
                        packageIdentifier = ManifestData.packageIdentifier,
                        packageVersion = ManifestData.packageVersion,
                        updateState = ManifestData.updateState
                    ).also { success("Pull request created: ${it.htmlUrl}") }
                    ManifestResultOption.WriteToFiles -> FileWriter.writeFiles(files, currentContext.terminal)
                    else -> return@also
                }
            }
        } else {
            FileWriter.writeFilesToDirectory(
                directory = System.getProperty("user.dir").toPath() / "${ManifestData.packageIdentifier} version ${ManifestData.packageVersion}",
                files = files,
                terminal = currentContext.terminal
            )
        }
    }

    private suspend fun Terminal.loopThroughInstallers(parameterUrls: Set<Url>? = null) = with(ManifestData) {
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls)
        } else if (Environment.isCI) {
            throw CliktError(colors.danger("No installers have been provided"), statusCode = 1)
        } else {
            val previousInstallerManifest = PreviousManifestData.installerManifest
            previousInstallerManifest?.installers?.sortedWith(installerSorter)?.forEachIndexed { index, installer ->
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
                installerDownloadPrompt()
                if (!skipAddInstaller) InstallerManifestData.addInstaller() else skipAddInstaller = false
            }
        }
    }

    private suspend fun Terminal.loopParameterUrls(parameterUrls: Set<Url>) = with(ManifestData) {
        val previousInstallerManifest = PreviousManifestData.installerManifest as InstallerManifest
        val previousInstallers = previousInstallerManifest.installers
        val previousUrls = previousInstallers.map(InstallerManifest.Installer::installerUrl)
        UrlsToInstallerMatcher.assertUniqueUrlsCount(parameterUrls, previousUrls.toSet(), colors)
        UrlsToInstallerMatcher.assertUrlsValid(parameterUrls, colors)
        val installerResults = mutableListOf<InstallerManifest.Installer>()
        val progressList = parameterUrls.map { url -> getDownloadProgressBar(url).apply(ProgressAnimation::start) }
        parameterUrls.forEachIndexed { index, url ->
            gitHubDetection = parameterUrls
                .firstOrNull { it.host.equals(GitHubDetection.gitHubWebsite, ignoreCase = true) }
                ?.let(::GitHubDetection)
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
            installerResults.sortedWith(installerSorter),
            previousInstallers
                .sortedWith(installerSorter)
                .map {
                    it.copy(
                        installerType = previousInstallerManifest.installerType ?: it.installerType,
                        scope = previousInstallerManifest.scope ?: it.scope
                    )
                }
        ).forEach { (_, newInstaller) ->
            architecture = newInstaller.architecture
            installerUrl = newInstaller.installerUrl
            installerSha256 = newInstaller.installerSha256.uppercase()
            upgradeBehavior = newInstaller.upgradeBehavior
            releaseDate = newInstaller.releaseDate
            scope = newInstaller.scope
            installerType = newInstaller.installerType
            InstallerManifestData.addInstaller()
        }
    }

    private suspend fun createFiles(
        packageIdentifier: String,
        packageVersion: String,
        defaultLocale: String?
    ): Map<String, String> {
        val allLocale = additionalMetadata?.locales?.find { it.name.equals("all", ignoreCase = true) }
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(manifestOverride),
            GitHubUtils.getDefaultLocaleManifestName(packageIdentifier, defaultLocale) to DefaultLocaleManifestData.createDefaultLocaleManifest(
                manifestOverride
            ),
            GitHubUtils.getVersionManifestName(packageIdentifier) to VersionManifestData.createVersionManifest()
        ) + PreviousManifestData.remoteLocaleData?.map { localeManifest ->
            val currentLocaleMetadata = additionalMetadata?.locales
                ?.find { it.name.equals(localeManifest.packageLocale, ignoreCase = true) }
            GitHubUtils.getLocaleManifestName(packageIdentifier, localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                manifestVersion = manifestOverride ?: Schemas.manifestVersion,
                releaseNotes = allLocale?.releaseNotes ?: currentLocaleMetadata?.releaseNotes,
                releaseNotesUrl = allLocale?.releaseNotesUrl ?: currentLocaleMetadata?.releaseNotesUrl,
                documentations = allLocale?.documentations
                    ?: currentLocaleMetadata?.documentations
                    ?: localeManifest.documentations
            ).toString()
        }.orEmpty()
    }
}
