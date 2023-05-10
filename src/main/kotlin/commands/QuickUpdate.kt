package commands

import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.clikt.parameters.options.validate
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.VersionManifestData
import data.installer.InstallerScope
import data.installer.InstallerType
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
import detection.ParameterUrls
import detection.github.GitHubDetection
import extensions.hash
import extensions.versionStringComparator
import input.FileWriter
import input.ManifestResultOption
import input.Prompts.pullRequestPrompt
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import okio.FileSystem
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import schemas.AdditionalMetadata
import schemas.Schemas
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import token.Token
import token.TokenStore
import utils.FileAnalyser
import utils.GitHubUtils
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.ManifestUtils.updateVersionInString
import utils.findArchitecture
import utils.findScope
import java.io.File

class QuickUpdate : CliktCommand(name = "update") {
    private val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    private val packageIdentifierParam: String? by option("--id", "--package-identifier")
    private val packageVersionParam: String? by option("--version", "--package-version")
    private val urls: List<Url>? by option().convert { Url(it) }.split(",")
    private lateinit var microsoftWingetPkgs: GHRepository
    private val manifestOverride: String? by option().validate {
        require("^\\d+\\.\\d+\\.\\d+$".toRegex() matches it) { "Manifest version must be in the format X.X.X" }
    }
    private val submit: Boolean by option().flag(default = false)
    private val tokenParameter: String? by option("-t", "--token", envvar = "GITHUB_TOKEN").validate {
        require(GitHub.connectUsingOAuth(it).isCredentialValid) {
            colors.danger("The token is invalid or has expired")
        }
    }
    private val additionalMetadata by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }
    private val fileSystem = FileSystem.SYSTEM

    override fun run(): Unit = runBlocking {
        val terminal = currentContext.terminal
        tokenParameter?.let { TokenStore.useTokenParameter(it) }
        with(AllManifestData) {
            if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
            if (isCIEnvironment) {
                info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
            }
            packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
            if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(terminal)
            microsoftWingetPkgs = GitHubImpl.microsoftWinGetPkgs
            allVersions = GitHubUtils.getAllVersions(microsoftWingetPkgs, packageIdentifier)
                ?.also { info("Found $packageIdentifier in the winget-pkgs repository") }
                ?: throw doesNotExistError(packageIdentifier, isUpdate = true, colors = colors)
            val latestVersion = (allVersions as List<String>).maxWithOrNull(versionStringComparator)
            info("Found latest version: $latestVersion")
            PreviousManifestData.init(packageIdentifier, latestVersion, microsoftWingetPkgs)
            packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
            GitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = terminal
            )
            updateState = getUpdateState(packageIdentifier, packageVersion, latestVersion)
            terminal.loopThroughInstallers(parameterUrls = urls?.toSet(), isCIEnvironment = isCIEnvironment)
            val files = createFiles(packageIdentifier, packageVersion, defaultLocale)
            for (manifest in files.values) {
                formattedManifestLinesSequence(manifest, colors).forEach(::echo)
            }
            if (submit) {
                GitHubImpl.commitAndPullRequest(
                    GitHubImpl.getWingetPkgsFork(terminal),
                    files = files,
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    updateState = updateState
                ).also { success("Pull request created: ${it.htmlUrl}") }
            } else if (!isCIEnvironment) {
                terminal.pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> {
                            GitHubImpl.commitAndPullRequest(
                                GitHubImpl.getWingetPkgsFork(terminal),
                                files = files,
                                packageIdentifier = packageIdentifier,
                                packageVersion = packageVersion,
                                updateState = updateState
                            ).also { success("Pull request created: ${it.htmlUrl}") }
                        }
                        ManifestResultOption.WriteToFiles -> FileWriter.writeFiles(files, terminal)
                        else -> return@also
                    }
                }
            } else {
                FileWriter.writeFilesToDirectory(
                    directory = File(System.getProperty("user.dir"), "$packageIdentifier version $packageVersion"),
                    files = files,
                    terminal = terminal
                )
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: Set<Url>? = null,
        isCIEnvironment: Boolean = false
    ) = with(AllManifestData) {
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls)
        } else if (isCIEnvironment) {
            throw CliktError(colors.danger("${Errors.error} No installers have been provided"), statusCode = 1)
        } else {
            val previousInstallerManifest = PreviousManifestData.installerManifest
            previousInstallerManifest?.installers?.forEachIndexed { index, installer ->
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

    private suspend fun Terminal.loopParameterUrls(parameterUrls: Set<Url>) = with(AllManifestData) {
        val previousInstallerManifest = PreviousManifestData.installerManifest as InstallerManifest
        val previousInstallers = previousInstallerManifest.installers
        val previousUrls = previousInstallers.map(InstallerManifest.Installer::installerUrl)
        ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls.toSet(), colors)
        ParameterUrls.assertUrlsValid(parameterUrls, colors)
        val installerResults = mutableListOf<InstallerManifest.Installer>()
        val progressList = parameterUrls.map { url -> getDownloadProgressBar(url).apply(ProgressAnimation::start) }
        parameterUrls.forEachIndexed { index, url ->
            gitHubDetection = parameterUrls
                .firstOrNull { it.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true) }
                ?.let(::GitHubDetection)
            val downloadedFile = Http.client.downloadFile(url, packageIdentifier, packageVersion, progressList[index], fileSystem)
            val fileAnalyser = FileAnalyser(downloadedFile.path, fileSystem)
            installerResults += try {
                InstallerManifest.Installer(
                    architecture = url.findArchitecture() ?: fileAnalyser.architecture,
                    installerType = fileAnalyser.installerType,
                    scope = url.findScope(),
                    installerSha256 = downloadedFile.path.hash(fileSystem),
                    signatureSha256 = fileAnalyser.signatureSha256,
                    installerUrl = url,
                    productCode = fileAnalyser.productCode,
                    upgradeBehavior = fileAnalyser.upgradeBehaviour,
                    releaseDate = gitHubDetection?.releaseDate ?: downloadedFile.lastModified
                )
            } finally {
                with(downloadedFile) {
                    fileSystem.delete(path)
                    removeFileDeletionHook()
                }
            }
        }
        progressList.forEach(ProgressAnimation::clear)
        ParameterUrls.matchInstallers(
            installerResults,
            previousInstallers.map {
                it.copy(
                    installerType = previousInstallerManifest.installerType ?: it.installerType,
                    scope = previousInstallerManifest.scope ?: it.scope
                )
            }
        ).forEach { (previousInstaller, newInstaller) ->
            installers += previousInstaller.copy(
                installerUrl = newInstaller.installerUrl,
                installerSha256 = newInstaller.installerSha256.uppercase(),
                signatureSha256 = newInstaller.signatureSha256?.uppercase(),
                productCode = newInstaller.productCode,
                releaseDate = newInstaller.releaseDate,
                nestedInstallerFiles = previousInstaller.nestedInstallerFiles?.map {
                    it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
                }
            )
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
                documentations = allLocale?.documentations ?: currentLocaleMetadata?.documentations
            ).toString()
        }.orEmpty()
    }
}
