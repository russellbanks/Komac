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
import schemas.Schema
import schemas.Schemas
import schemas.manifest.EncodeConfig
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import utils.GitHubUtils
import utils.ManifestUtils.formattedManifestLinesSequence

class NewManifest : CliktCommand(name = "new") {
    private val manifestOverride: String? by option().validate {
        require("^\\d+\\.\\d+\\.\\d+$".toRegex() matches it) { "Manifest version must be in the format X.X.X" }
    }

    @OptIn(ExperimentalStdlibApi::class)
    override fun run(): Unit = runBlocking {
        with(AllManifestData) {
            if (manifestOverride != null) Schemas.manifestVersion = manifestOverride as String
            if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
            packageIdentifier = prompt(PackageIdentifier)
            if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)
            val latestVersion = allVersions?.maxWithOrNull(versionStringComparator)
            if (latestVersion != null) {
                info("Found $packageIdentifier in the winget-pkgs repository")
                info("Found latest version: $latestVersion")
            }
            PreviousManifestData.init(packageIdentifier, latestVersion, GitHubImpl.microsoftWinGetPkgs)
            packageVersion = prompt(PackageVersion)
            GitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            updateState = getUpdateState(packageIdentifier, packageVersion, latestVersion)
            do {
                currentContext.terminal.installerDownloadPrompt()
                installerType = installerType ?: prompt(InstallerType)
                for (switch in Switch.entries) {
                    InstallerSwitch.installerSwitchPrompt(switch, currentContext.terminal)
                }
                installerLocale = msi?.productLanguage ?: prompt(Locale.Installer)
                InstallerScope.installerScopePrompt(currentContext.terminal)
                upgradeBehavior = prompt(UpgradeBehaviour)
                if (!skipAddInstaller) {
                    InstallerManifestData.addInstaller()
                } else {
                    skipAddInstaller = false
                }
                val loop = confirm(colors.info(additionalInstallerInfo)) ?: throw ProgramResult(ExitCode.CtrlC)
            } while (loop)
            fileExtensions = prompt(FileExtensions)
            protocols = prompt(Protocols)
            commands = prompt(Commands)
            installerSuccessCodes = prompt(InstallerSuccessCodes)
            installModes = prompt(InstallModes)
            defaultLocale = prompt(Locale.Package)
            publisher = msi?.manufacturer ?: msix?.publisherDisplayName ?: prompt(Publisher)
            packageName = msix?.displayName ?: prompt(PackageName)
            moniker = prompt(Moniker)
            publisherUrl = gitHubDetection?.publisherUrl ?: prompt(Publisher.Url)
            author = prompt(Author)
            packageUrl = gitHubDetection?.packageUrl ?: prompt(PackageUrl)
            license = gitHubDetection?.license ?: prompt(License)
            licenseUrl = gitHubDetection?.licenseUrl ?: prompt(License.Url)
            copyright = prompt(Copyright)
            copyrightUrl = prompt(Copyright.Url)
            tags = gitHubDetection?.topics ?: prompt(Tags)
            shortDescription = if (gitHubDetection?.shortDescription != null && PreviousManifestData.defaultLocaleManifest?.shortDescription != null) {
                gitHubDetection?.shortDescription
            } else {
                prompt(Description.Short)
            }
            description = prompt(Description.Long)
            releaseNotesUrl = gitHubDetection?.releaseNotesUrl ?: prompt(ReleaseNotesUrl)
            val files = createFiles()
            for (manifest in files.values) {
                formattedManifestLinesSequence(manifest, colors).forEach(::echo)
            }
            when (currentContext.terminal.pullRequestPrompt(packageIdentifier, packageVersion)) {
                ManifestResultOption.PullRequest -> {
                    GitHubImpl.commitAndPullRequest(
                        wingetPkgsFork = GitHubImpl.getWingetPkgsFork(currentContext.terminal),
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

    private suspend fun createFiles(): Map<String, String> = with(AllManifestData) {
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(),
            GitHubUtils.getDefaultLocaleManifestName(
                identifier = packageIdentifier,
                defaultLocale = defaultLocale
            ) to DefaultLocaleManifestData.createDefaultLocaleManifest(),
            GitHubUtils.getVersionManifestName(packageIdentifier) to VersionManifestData.createVersionManifest()
        ) + PreviousManifestData.remoteLocaleData?.map { localeManifest ->
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
                        manifestVersion = Schemas.manifestVersion
                    )
                )
            )
        }.orEmpty()
    }

    companion object {
        private const val additionalInstallerInfo = "Do you want to create another installer?"
    }
}
