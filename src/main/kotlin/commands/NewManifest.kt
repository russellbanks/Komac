package commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.option
import data.DefaultLocaleManifestData
import data.InstallerManifestData
import data.ManifestData
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
import github.GitHubImpl
import github.GitHubUtils
import io.ExitCode
import io.FileWriter.writeFiles
import io.ManifestResultOption
import io.Switch
import io.menu.radioMenu
import kotlinx.coroutines.runBlocking
import schemas.Schemas
import schemas.manifest.InstallerManifest
import schemas.manifest.Schema
import token.Token
import token.TokenStore
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.versionStringComparator

class NewManifest : CliktCommand(name = "new") {
    private val packageIdentifierParam: String? by option(
        "-i", "--id", "--package-identifier",
        help = "Package identifier. Example: Publisher.Package"
    )

    private val packageVersionParam: String? by option(
        "-v", "--version", "--package-version",
        help = "Package version. Example: 1.2.3"
    )

    private val manifestOverride: String? by option(
        "--manifest-version", "--manifest-override",
        help = "Overrides the manifest version.",
        envvar = "MANIFEST_VERSION"
    ).check { Regex(Schemas.manifestVersionRegex) matches it }

    @OptIn(ExperimentalStdlibApi::class)
    override fun run(): Unit = runBlocking {
        with(ManifestData) {
            if (manifestOverride != null) Schemas.manifestVersion = manifestOverride as String
            if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
            packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
            if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)
            val latestVersion = allVersions?.maxWithOrNull(versionStringComparator)
            if (latestVersion != null) {
                info("Found $packageIdentifier in the winget-pkgs repository")
                info("Found latest version: $latestVersion")
            }
            PreviousManifestData.init(packageIdentifier, latestVersion, GitHubImpl.microsoftWinGetPkgs)
            packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
            GitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            updateState = getUpdateState(packageIdentifier, packageVersion, latestVersion)
            do {
                currentContext.terminal.installerDownloadPrompt()
                installerType = installerType ?: prompt(InstallerType)
                if (installerType == InstallerManifest.InstallerType.EXE) {
                    installerSwitches[Switch.Silent] = prompt(InstallerSwitch.Silent)
                    installerSwitches[Switch.SilentWithProgress] = prompt(InstallerSwitch.SilentWithProgress)
                }
                installerSwitches[Switch.Custom] = prompt(InstallerSwitch.Custom)
                installerLocale = msi?.productLanguage ?: prompt(Locale.Installer)
                if (scope == null && installerType != InstallerManifest.InstallerType.PORTABLE) {
                    scope = prompt(InstallerScope)
                }
                upgradeBehavior = prompt(UpgradeBehaviour)
                if (!skipAddInstaller) InstallerManifestData.addInstaller() else skipAddInstaller = false
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
            for (manifest in files.values.map(Schema::toString)) {
                formattedManifestLinesSequence(manifest, colors).forEach(::echo)
            }
            info("What would you like to do with $packageIdentifier $packageVersion?")
            currentContext.terminal.radioMenu<ManifestResultOption> {
                items = ManifestResultOption.entries
                default = ManifestResultOption.PullRequest
            }.prompt().also { manifestResultOption ->
                when (manifestResultOption) {
                    ManifestResultOption.PullRequest -> {
                        GitHubImpl.commitAndPullRequest(
                            wingetPkgsFork = GitHubImpl.getWingetPkgsFork(currentContext.terminal),
                            files = files,
                            packageIdentifier = packageIdentifier,
                            packageVersion = packageVersion,
                            updateState = updateState,
                            terminal = currentContext.terminal
                        ).also { success("Pull request created: ${it.htmlUrl}") }
                    }
                    ManifestResultOption.WriteToFiles -> writeFiles(files, currentContext.terminal)
                    else -> return@runBlocking
                }
            }
        }
    }

    private suspend fun createFiles(): Map<String, Schema> = with(ManifestData) {
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier)
                    to InstallerManifestData.createInstallerManifest(manifestOverride),
            GitHubUtils.getDefaultLocaleManifestName(identifier = packageIdentifier, defaultLocale = defaultLocale)
                    to DefaultLocaleManifestData.createDefaultLocaleManifest(manifestOverride),
            GitHubUtils.getVersionManifestName(packageIdentifier)
                    to VersionManifestData.createVersionManifest(manifestOverride)
        ) + PreviousManifestData.remoteLocaleData?.map { localeManifest ->
            GitHubUtils.getLocaleManifestName(packageIdentifier, localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                manifestVersion = Schemas.manifestVersion
            )
        }.orEmpty()
    }

    companion object {
        private const val additionalInstallerInfo = "Do you want to create another installer?"
    }
}
