package commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.validate
import data.AllManifestData
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.VersionManifestData
import data.installer.Commands
import data.installer.FileExtensions
import data.installer.InstallModes
import data.installer.InstallerScope
import data.installer.InstallerSuccessCodes
import data.installer.InstallerSwitch
import data.installer.InstallerType
import data.installer.Protocols
import data.installer.UpgradeBehaviour
import data.locale.Author
import data.locale.Copyright
import data.locale.Description
import data.locale.License
import data.locale.Moniker
import data.locale.PackageUrl
import data.locale.ReleaseNotesUrl
import data.locale.Tags
import data.shared.Locale
import data.shared.PackageIdentifier
import data.shared.PackageName
import data.shared.PackageVersion
import data.shared.Publisher
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
import extensions.versionStringComparator
import input.ExitCode
import input.FileWriter.writeFiles
import input.ManifestResultOption
import input.Prompts.pullRequestPrompt
import input.Switch
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import network.Http
import schemas.Schema
import schemas.Schemas
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import utils.GitHubUtils
import utils.ManifestUtils.formattedManifestLinesSequence

class NewManifest : CliktCommand(name = "new") {
    private val tokenStore = TokenStore()
    private val allManifestData = AllManifestData()
    private var previousManifestData: PreviousManifestData? = null
    private val client = Http.client
    private val gitHubImpl by lazy { GitHubImpl(tokenStore.token as String, client) }
    private val manifestOverride: String? by option().validate {
        require("^\\d+\\.\\d+\\.\\d+$".toRegex() matches it) { "Manifest version must be in the format X.X.X" }
    }
    private val previousInstallerManifest: InstallerManifest?
        get() = previousManifestData?.remoteInstallerData
    private val defaultLocaleManifest: DefaultLocaleManifest?
        get() = previousManifestData?.remoteDefaultLocaleData

    override fun run(): Unit = runBlocking {
        with(allManifestData) {
            if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
            packageIdentifier = prompt(PackageIdentifier)
            if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(gitHubImpl.getMicrosoftWinGetPkgs(), packageIdentifier)
            val latestVersion = allVersions?.maxWithOrNull(versionStringComparator)
            if (latestVersion != null) {
                info("Found $packageIdentifier in the winget-pkgs repository")
                info("Found latest version: $latestVersion")
            }
            launch {
                if (latestVersion != null) {
                    previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, gitHubImpl.getMicrosoftWinGetPkgs())
                }
            }
            packageVersion = prompt(PackageVersion)
            gitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            updateState = getUpdateState(packageIdentifier, packageVersion, latestVersion, gitHubImpl)
            do {
                currentContext.terminal.installerDownloadPrompt(allManifestData, client, gitHubImpl)
                installerType = installerType ?: prompt(InstallerType(previousInstallerManifest, installers.size))
                Switch.values().forEach { InstallerSwitch(allManifestData, previousInstallerManifest).installerSwitchPrompt(it, currentContext.terminal) }
                installerLocale = msi?.productLanguage ?: prompt(Locale.Installer(previousInstallerManifest, installers.size))
                InstallerScope(allManifestData, previousInstallerManifest).installerScopePrompt(currentContext.terminal)
                upgradeBehavior = prompt(UpgradeBehaviour(allManifestData, previousInstallerManifest))
                if (!skipAddInstaller) {
                    InstallerManifestData.addInstaller(allManifestData, previousInstallerManifest, defaultLocaleManifest)
                } else {
                    skipAddInstaller = false
                }
                val loop = confirm(colors.info(additionalInstallerInfo)) ?: throw ProgramResult(ExitCode.CtrlC)
            } while (loop)
            fileExtensions = prompt(FileExtensions(previousInstallerManifest, installers.size))
            protocols = prompt(Protocols(previousInstallerManifest, installers.size))
            commands = prompt(Commands(previousInstallerManifest, installers.size))
            installerSuccessCodes = prompt(InstallerSuccessCodes(previousInstallerManifest, installers.size))
            installModes = prompt(InstallModes(previousInstallerManifest, installers.size))
            defaultLocale = prompt(Locale.Package(defaultLocaleManifest?.packageLocale))
            publisher = msi?.manufacturer ?: msix?.publisherDisplayName ?: prompt(Publisher(defaultLocaleManifest?.publisher))
            packageName = msix?.displayName ?: prompt(PackageName(msi, defaultLocaleManifest?.packageName))
            moniker = prompt(Moniker(defaultLocaleManifest?.moniker))
            publisherUrl = gitHubDetection?.publisherUrl
                ?: prompt(Publisher.Url(defaultLocaleManifest?.publisherUrl, client))
            author = prompt(Author(defaultLocaleManifest?.author))
            packageUrl = gitHubDetection?.packageUrl ?: prompt(PackageUrl(defaultLocaleManifest?.packageUrl, client))
            license = gitHubDetection?.license ?: prompt(License(defaultLocaleManifest?.license))
            licenseUrl = gitHubDetection?.licenseUrl ?: prompt(License.Url(defaultLocaleManifest?.licenseUrl, client))
            copyright = prompt(Copyright(defaultLocaleManifest?.copyright))
            copyrightUrl = prompt(Copyright.Url(defaultLocaleManifest?.copyrightUrl, client))
            tags = gitHubDetection?.topics ?: prompt(Tags(defaultLocaleManifest?.tags))
            shortDescription = if (gitHubDetection?.shortDescription != null && defaultLocaleManifest?.shortDescription != null) {
                gitHubDetection?.shortDescription
            } else {
                prompt(Description.Short(msix))
            }
            description = prompt(Description.Long(defaultLocaleManifest?.description))
            releaseNotesUrl = gitHubDetection?.releaseNotesUrl ?: prompt(ReleaseNotesUrl(client))
            val files = createFiles()
            files.values.forEach { manifest -> formattedManifestLinesSequence(manifest, colors).forEach(::echo) }
            when (currentContext.terminal.pullRequestPrompt(packageIdentifier, packageVersion)) {
                ManifestResultOption.PullRequest -> {
                    gitHubImpl.commitAndPullRequest(
                        wingetPkgsFork = gitHubImpl.getWingetPkgsFork(currentContext.terminal),
                        files = files,
                        packageIdentifier = packageIdentifier,
                        packageVersion = packageVersion,
                        updateState = updateState
                    ).also { success("Pull request created: ${it.htmlUrl}") }
                }
                ManifestResultOption.WriteToFiles -> writeFiles(files, currentContext.terminal)
                else -> return@runBlocking
            }
        }
    }

    private suspend fun createFiles(): Map<String, String> = with(allManifestData) {
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(
                allManifestData = allManifestData,
                previousInstallerManifest = previousInstallerManifest,
                manifestOverride = manifestOverride
            ),
            GitHubUtils.getDefaultLocaleManifestName(
                identifier = packageIdentifier,
                defaultLocale = defaultLocale,
                previousDefaultLocale = defaultLocaleManifest?.packageLocale
            ) to DefaultLocaleManifestData.createDefaultLocaleManifest(
                allManifestData = allManifestData,
                previousManifestData = previousManifestData,
                manifestOverride = manifestOverride
            ),
            GitHubUtils.getVersionManifestName(packageIdentifier) to VersionManifestData.createVersionManifest(
                allManifestData = allManifestData,
                manifestOverride = manifestOverride,
                previousVersionData = previousManifestData?.previousVersionData
            )
        ) + previousManifestData?.remoteLocaleData?.map { localeManifest ->
            GitHubUtils.getLocaleManifestName(
                packageIdentifier,
                localeManifest.packageLocale
            ) to Schemas.buildManifestString(
                Schema.Locale,
                EncodeConfig.yamlDefault.encodeToString(
                    LocaleManifest.serializer(),
                    localeManifest.copy(
                        packageIdentifier = packageIdentifier,
                        packageVersion = packageVersion,
                        manifestVersion = manifestOverride ?: Schemas.manifestVersion
                    )
                )
            )
        }.orEmpty()
    }

    companion object {
        private const val additionalInstallerInfo = "Do you want to create another installer?"
    }
}
