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
import data.installer.InstallerType
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import data.shared.PackageVersion.getHighestVersion
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
import detection.ParameterUrls
import detection.github.GitHubDetection
import extensions.GitHubExtensions.printResultTo
import input.FileWriter
import input.ManifestResultOption
import input.Prompts.pullRequestPrompt
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils.downloadFile
import network.HttpUtils.getDownloadProgressBar
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import schemas.AdditionalMetadata
import schemas.Schema
import schemas.Schemas
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import utils.FileAnalyser
import utils.GitHubUtils
import utils.Hashing.hash
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.ManifestUtils.updateVersionInString
import utils.findArchitecture
import utils.findScope
import java.io.File

class QuickUpdate : CliktCommand(name = "update") {
    private val allManifestData = AllManifestData()
    private val tokenStore = TokenStore()
    private lateinit var previousManifestData: PreviousManifestData
    private val previousInstallerManifest: InstallerManifest
        get() = previousManifestData.remoteInstallerData
            ?: throw CliktError(colors.danger("Failed to retrieve previous installers"), statusCode = 1)
    private val client = Http.client
    private val gitHubImpl by lazy { GitHubImpl(tokenStore.token as String, client) }
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
    private val additionalMetadataParam by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }

    override fun run(): Unit = runBlocking {
        val terminal = currentContext.terminal
        tokenParameter?.let { tokenStore.useTokenParameter(it) }
        with(allManifestData) {
            if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
            if (isCIEnvironment) {
                info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
            }
            packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
            if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(terminal)
            microsoftWingetPkgs = gitHubImpl.getMicrosoftWinGetPkgs()
            allVersions = GitHubUtils.getAllVersions(microsoftWingetPkgs, packageIdentifier)
                ?.also { info("Found $packageIdentifier in the winget-pkgs repository") }
                ?: throw doesNotExistError(packageIdentifier, isUpdate = true, colors = colors)
            val latestVersion = (allVersions as List<String>).getHighestVersion()
            info("Found latest version: $latestVersion")
            previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, microsoftWingetPkgs)
            packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
            gitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = terminal
            )
            updateState = getUpdateState(packageIdentifier, packageVersion, latestVersion, gitHubImpl)
            terminal.loopThroughInstallers(parameterUrls = urls?.toSet(), isCIEnvironment = isCIEnvironment)
            val files = createFiles(packageIdentifier, packageVersion, defaultLocale)
            files.values.forEach { manifest -> formattedManifestLinesSequence(manifest, colors).forEach(::echo) }
            if (submit) {
                gitHubImpl.commitAndPullRequest(
                    gitHubImpl.getWingetPkgsFork(terminal),
                    files = files,
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    updateState = updateState
                ) printResultTo terminal
            } else if (!isCIEnvironment) {
                terminal.pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> {
                            gitHubImpl.commitAndPullRequest(
                                gitHubImpl.getWingetPkgsFork(terminal),
                                files = files,
                                packageIdentifier = packageIdentifier,
                                packageVersion = packageVersion,
                                updateState = updateState
                            ) printResultTo terminal
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
    ) = with(allManifestData) {
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls = parameterUrls, previousInstallerManifest = previousInstallerManifest)
        } else if (isCIEnvironment) {
            throw CliktError(colors.danger("${Errors.error} No installers have been provided"), statusCode = 1)
        } else {
            previousInstallerManifest.installers.forEachIndexed { index, installer ->
                info("Installer Entry ${index.inc()}/${previousInstallerManifest.installers.size}")
                listOf(
                    InstallerManifest.Installer.Architecture::class.simpleName to installer.architecture,
                    InstallerType::class.simpleName to (installer.installerType ?: previousInstallerManifest.installerType),
                    InstallerManifest.Scope::class.simpleName to (installer.scope ?: previousInstallerManifest.scope),
                    InstallerManifest::installerLocale::class.simpleName to
                        (installer.installerLocale ?: previousInstallerManifest.installerLocale)
                ).forEach { (promptType, value) ->
                    value?.let {
                        echo("  $promptType: ${colors.info(it.toString())}")
                    }
                }
                echo()
                installerDownloadPrompt(allManifestData, client, gitHubImpl)
                if (!skipAddInstaller) {
                    InstallerManifestData.addInstaller(
                        allManifestData, previousInstallerManifest, previousManifestData.remoteDefaultLocaleData
                    )
                } else {
                    skipAddInstaller = false
                }
            }
        }
    }

    private suspend fun Terminal.loopParameterUrls(
        parameterUrls: Set<Url>,
        previousInstallerManifest: InstallerManifest
    ) = with(allManifestData) {
        val previousInstallers = previousInstallerManifest.installers
        val previousUrls = previousInstallers.map(InstallerManifest.Installer::installerUrl)
        ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls.toSet(), colors)
        ParameterUrls.assertUrlsValid(parameterUrls, client, colors)
        val installerResults = mutableListOf<InstallerManifest.Installer>()
        val progressList = parameterUrls.map { url -> getDownloadProgressBar(url).apply(ProgressAnimation::start) }
        parameterUrls.forEachIndexed { index, url ->
            gitHubDetection = parameterUrls
                .firstOrNull { it.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true) }
                ?.let { GitHubDetection(it, gitHubImpl, client) }
            val downloadedFile = client.downloadFile(url, packageIdentifier, packageVersion, progressList[index])
            val fileAnalyser = FileAnalyser(downloadedFile.file)
            installerResults += try {
                InstallerManifest.Installer(
                    architecture = url.findArchitecture() ?: fileAnalyser.getArchitecture(),
                    installerType = fileAnalyser.getInstallerType(),
                    scope = url.findScope(),
                    installerSha256 = downloadedFile.file.hash(),
                    signatureSha256 = fileAnalyser.getSignatureSha256(),
                    installerUrl = url,
                    productCode = fileAnalyser.getProductCode(),
                    upgradeBehavior = fileAnalyser.getUpgradeBehaviour(),
                    releaseDate = gitHubDetection?.releaseDate ?: downloadedFile.lastModified
                )
            } finally {
                with(downloadedFile) {
                    file.delete()
                    removeFileDeletionHook()
                }
            }
        }
        progressList.forEach(ProgressAnimation::clear)
        ParameterUrls.matchInstallers(
            installerResults,
            previousInstallers.map {
                it.copy(
                    installerType = previousInstallerManifest.installerType?.toPerInstallerType() ?: it.installerType,
                    scope = previousInstallerManifest.scope?.toPerScopeInstallerType() ?: it.scope
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
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(allManifestData, previousInstallerManifest, manifestOverride),
            GitHubUtils.getDefaultLocaleManifestName(packageIdentifier, defaultLocale, previousManifestData.remoteDefaultLocaleData?.packageLocale) to DefaultLocaleManifestData.createDefaultLocaleManifest(allManifestData, previousManifestData, manifestOverride),
            GitHubUtils.getVersionManifestName(packageIdentifier) to VersionManifestData.createVersionManifest(
                allManifestData = allManifestData,
                manifestOverride = manifestOverride,
                previousVersionData = previousManifestData.previousVersionData
            )
        ) + previousManifestData.remoteLocaleData?.mapNotNull { localeManifest ->
            additionalMetadataParam?.locales
                ?.find { it.name.equals(other = localeManifest.packageLocale, ignoreCase = true) }
                ?.let { metadataCurrentLocale ->
                    val allLocale = additionalMetadataParam?.locales?.find {
                        it.name.equals(other = "all", ignoreCase = true)
                    }
                    GitHubUtils.getLocaleManifestName(packageIdentifier, localeManifest.packageLocale) to run {
                        val copyLocaleManifest = localeManifest.copy(
                            packageIdentifier = packageIdentifier,
                            packageVersion = packageVersion,
                            manifestVersion = manifestOverride ?: Schemas.manifestVersion,
                            releaseNotes = allLocale?.releaseNotes ?: metadataCurrentLocale.releaseNotes,
                            releaseNotesUrl = allLocale?.releaseNotesUrl ?: metadataCurrentLocale.releaseNotesUrl,
                            documentations = allLocale?.documentations ?: metadataCurrentLocale.documentations
                        )
                        Schemas.buildManifestString(
                            schema = Schema.Locale,
                            rawString = EncodeConfig.yamlDefault.encodeToString(
                                serializer = LocaleManifest.serializer(),
                                value = copyLocaleManifest
                            )
                        )
                    }
                }
        }.orEmpty()
    }
}
