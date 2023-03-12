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
import data.VersionUpdateState
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
import data.shared.PackageVersion.getHighestVersion
import data.shared.Publisher
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
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
import schemas.manifest.EncodeConfig
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import utils.GitHubUtils
import utils.ManifestUtils

class NewManifest : CliktCommand(name = "new") {
    private val tokenStore = TokenStore()
    private val allManifestData = AllManifestData()
    private var previousManifestData: PreviousManifestData? = null
    private val client = Http.client
    private val gitHubImpl by lazy { GitHubImpl(tokenStore.token as String, client) }
    private val manifestOverride: String? by option().validate {
        require("^\\d+\\.\\d+\\.\\d+$".toRegex() matches it) { "Manifest version must be in the format X.X.X" }
    }
    private val previousInstallerManifest = previousManifestData?.remoteInstallerData
    private val previousDefaultLocaleManifest = previousManifestData?.remoteDefaultLocaleData

    override fun run(): Unit = runBlocking {
        with(allManifestData) {
            if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
            packageIdentifier = prompt(PackageIdentifier)
            if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(gitHubImpl.getMicrosoftWinGetPkgs(), packageIdentifier)
            latestVersion = allVersions?.getHighestVersion()?.also {
                if (System.getenv("CI")?.toBooleanStrictOrNull() != true) {
                    info("Found $packageIdentifier in the winget-pkgs repository")
                    info("Found latest version: $it")
                }
            }
            launch {
                if (updateState != VersionUpdateState.NewPackage) {
                    previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, gitHubImpl.getMicrosoftWinGetPkgs())
                }
            }
            packageVersion = prompt(PackageVersion)
            gitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            updateState = getUpdateState(updateState, packageIdentifier, packageVersion, latestVersion, gitHubImpl)
            do {
                currentContext.terminal.installerDownloadPrompt(allManifestData, client, gitHubImpl)
                installerType = installerType ?: prompt(InstallerType(previousInstallerManifest?.await(), installers.size))
                Switch.values().forEach { InstallerSwitch(allManifestData, previousInstallerManifest?.await()).installerSwitchPrompt(it, currentContext.terminal) }
                installerLocale = prompt(Locale.Installer(allManifestData, previousManifestData?.remoteInstallerData?.await()))
                InstallerScope(allManifestData, previousInstallerManifest?.await()).installerScopePrompt(currentContext.terminal)
                upgradeBehavior = prompt(UpgradeBehaviour(allManifestData, previousInstallerManifest?.await()))
                if (!skipAddInstaller) {
                    InstallerManifestData.addInstaller(allManifestData, previousManifestData)
                } else {
                    skipAddInstaller = false
                }
                val loop = confirm(colors.info(additionalInstallerInfo)) ?: throw ProgramResult(ExitCode.CtrlC.code)
            } while (loop)
            fileExtensions = prompt(FileExtensions(previousInstallerManifest?.await(), installers.size))
            protocols = prompt(Protocols(previousInstallerManifest?.await(), installers.size))
            commands = prompt(Commands(previousInstallerManifest?.await(), installers.size))
            installerSuccessCodes = prompt(InstallerSuccessCodes(previousInstallerManifest?.await(), installers.size))
            installModes = prompt(InstallModes(previousInstallerManifest?.await(), installers.size))
            defaultLocale = prompt(Locale.Package(previousDefaultLocaleManifest?.await()?.packageLocale))
            publisher = prompt(Publisher(msi, msix, previousDefaultLocaleManifest?.await()?.publisher))
            packageName = prompt(PackageName(msi, msix, previousDefaultLocaleManifest?.await()?.packageName))
            moniker = prompt(Moniker(previousDefaultLocaleManifest?.await()?.moniker))
            publisherUrl = gitHubDetection?.publisherUrl
                ?: prompt(Publisher.Url(previousDefaultLocaleManifest?.await()?.publisherUrl, client))
            author = prompt(Author(previousDefaultLocaleManifest?.await()?.author))
            packageUrl = prompt(PackageUrl(gitHubDetection, previousDefaultLocaleManifest?.await()?.packageUrl, client))
            license = gitHubDetection?.license ?: prompt(License(previousDefaultLocaleManifest?.await()?.license))
            licenseUrl = gitHubDetection?.licenseUrl
                ?: prompt(License.Url(previousDefaultLocaleManifest?.await()?.licenseUrl, client))
            copyright = prompt(Copyright(previousDefaultLocaleManifest?.await()?.copyright))
            copyrightUrl = prompt(Copyright.Url(previousDefaultLocaleManifest?.await()?.copyrightUrl, client))
            tags = prompt(Tags(gitHubDetection, previousDefaultLocaleManifest?.await()?.tags))
            shortDescription = prompt(Description.Short(allManifestData, previousDefaultLocaleManifest?.await()?.shortDescription))
            description = prompt(Description.Long(previousDefaultLocaleManifest?.await()?.description))
            releaseNotesUrl = gitHubDetection?.releaseNotesUrl ?: prompt(ReleaseNotesUrl(client))
            val files = createFiles()
            files.values.forEach { manifest ->
                ManifestUtils.formattedManifestLinesSequence(manifest, colors).forEach(::echo)
            }
            currentContext.terminal.pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                when (manifestResultOption) {
                    ManifestResultOption.PullRequest -> {
                        gitHubImpl.commitAndPullRequest(
                            files = files,
                            packageIdentifier = packageIdentifier,
                            packageVersion = packageVersion,
                            updateState = updateState,
                            terminal = currentContext.terminal
                        )
                    }
                    ManifestResultOption.WriteToFiles -> writeFiles(files, currentContext.terminal)
                    else -> return@also
                }
            }
        }
    }

    private suspend fun createFiles(): Map<String, String> = with(allManifestData) {
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(
                allManifestData = allManifestData,
                previousInstallerManifest = previousInstallerManifest?.await(),
                manifestOverride = manifestOverride
            ),
            GitHubUtils.getDefaultLocaleManifestName(
                identifier = packageIdentifier,
                defaultLocale = packageVersion,
                previousDefaultLocale = previousDefaultLocaleManifest?.await()?.packageLocale
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
        ) + previousManifestData?.remoteLocaleData?.await()?.map { localeManifest ->
            GitHubUtils.getLocaleManifestName(packageIdentifier, localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                manifestVersion = manifestOverride ?: Schemas.manifestVersion
            ).let {
                Schemas.buildManifestString(
                    Schema.Locale,
                    EncodeConfig.yamlDefault.encodeToString(LocaleManifest.serializer(), it)
                )
            }
        }.orEmpty()
    }

    companion object {
        private const val additionalInstallerInfo = "Do you want to create another installer?"
    }
}
