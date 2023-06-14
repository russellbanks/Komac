package commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import data.InstallerManifestData
import data.PreviousManifestData
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
import data.shared.InstallerUrl
import data.shared.InstallerUrl.downloadInstaller
import data.shared.InstallerUrl.msixBundleDetection
import data.shared.Locale
import data.shared.PackageIdentifier
import data.shared.PackageName
import data.shared.PackageVersion
import data.shared.Publisher
import github.GitHubDetection
import github.GitHubImpl
import github.GitHubImpl.promptIfPullRequestExists
import github.GitHubUtils
import io.FileWriter.writeFiles
import io.ManifestResultOption
import io.ktor.http.Url
import io.menu.radioMenu
import io.menu.yesNoMenu
import kotlinx.coroutines.runBlocking
import network.WebPageScraper
import schemas.Schemas
import schemas.manifest.InstallerManifest
import schemas.manifest.Manifest
import token.Token
import token.TokenStore
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.versionStringComparator

class NewManifest : CliktCommand(name = "new") {
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

    private val manifestOverride: String by option(
        "--manifest-version", "--manifest-override",
        help = "Overrides the manifest version.",
        envvar = "MANIFEST_VERSION"
    ).default(Schemas.manifestVersion).check { Regex(Schemas.manifestVersionRegex) matches it }

    private var defaultLocale: String? = null
    private var license: String? = null
    private var shortDescription: String? = null
    private var moniker: String? = null
    private var publisherUrl: Url? = null
    private var author: String? = null
    private var packageUrl: Url? = null
    private var licenseUrl: Url? = null
    private var copyright: String? = null
    private var copyrightUrl: Url? = null
    private var commands: List<String>? = null
    private var fileExtensions: List<String>? = null
    private var protocols: List<String>? = null
    private var installerSuccessCodes: List<Long>? = null
    private var tags: List<String>? = null
    private var installModes: List<InstallerManifest.InstallModes>? = null
    private var installerType: InstallerManifest.InstallerType? = null
    private var installerSwitches: InstallerManifest.InstallerSwitches = InstallerManifest.InstallerSwitches()
    private var installers = emptyList<InstallerManifest.Installer>()
    private var upgradeBehavior: InstallerManifest.UpgradeBehavior? = null
    private var releaseNotesUrl: Url? = null
    private var description: String? = null

    private lateinit var packageIdentifier: String
    private lateinit var packageVersion: String
    private lateinit var publisher: String
    private lateinit var packageName: String

    @OptIn(ExperimentalStdlibApi::class)
    override fun run(): Unit = runBlocking {
        if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
        packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
        if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
        val allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)
        val latestVersion = allVersions?.maxWithOrNull(versionStringComparator)
        if (latestVersion != null) {
            info("Found $packageIdentifier in the winget-pkgs repository")
            info("Found latest version: $latestVersion")
        }
        val previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, GitHubImpl.microsoftWinGetPkgs)
        packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
        currentContext.terminal.promptIfPullRequestExists(identifier = packageIdentifier, version = packageVersion)
        val updateState = when {
            allVersions == null -> VersionUpdateState.NewPackage
            packageVersion in allVersions -> VersionUpdateState.UpdateVersion
            latestVersion != null && packageVersion == maxOf(
                packageVersion, latestVersion, versionStringComparator
            ) -> VersionUpdateState.NewVersion
            else -> VersionUpdateState.AddVersion
        }
        lateinit var downloadResult: InstallerUrl.DownloadResult
        lateinit var installerUrl: Url
        val previousInstallerManifest = previousManifestData.installerManifest.await()
        var gitHubDetection: GitHubDetection? = null
        do {
            var scope: InstallerManifest.Scope? = null
            installerUrl = prompt(InstallerUrl)
            if (installers.map(InstallerManifest.Installer::installerUrl).contains(installerUrl)) {
                installers += installers.first { it.installerUrl == installerUrl }
            } else {
                downloadResult = currentContext.terminal.downloadInstaller(
                    packageIdentifier,
                    packageVersion,
                    installerUrl
                )
                currentContext.terminal.msixBundleDetection(downloadResult.msixBundle)
                installerType = downloadResult.installerType ?: prompt(InstallerType(installerUrl, installers, previousInstallerManifest))
                if (installerType == InstallerManifest.InstallerType.EXE) {
                    installerSwitches[InstallerManifest.InstallerSwitches.Key.Silent] = prompt(InstallerSwitch.Silent(installers.size, previousInstallerManifest))
                    installerSwitches[InstallerManifest.InstallerSwitches.Key.SilentWithProgress] = prompt(InstallerSwitch.SilentWithProgress(installers.size, previousInstallerManifest))
                }
                installerSwitches[InstallerManifest.InstallerSwitches.Key.Custom] = prompt(InstallerSwitch.Custom(installers.size, previousInstallerManifest))
                val installerLocale = downloadResult.msi?.productLanguage ?: prompt(Locale.Installer(installers.size, previousInstallerManifest))
                if (downloadResult.scope == null && installerType != InstallerManifest.InstallerType.PORTABLE) {
                    scope = prompt(InstallerScope(installers.size, previousInstallerManifest))
                }
                upgradeBehavior = prompt(UpgradeBehaviour(installers.size, previousInstallerManifest))
                if (gitHubDetection == null && installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
                    gitHubDetection = GitHubDetection(installerUrl)
                }
                InstallerManifestData.addInstaller(
                    packageVersion = packageVersion,
                    installerUrl = installerUrl,
                    installerSha256 = downloadResult.installerSha256,
                    installerLocale = installerLocale,
                    installers = installers,
                    scope = scope,
                    architecture = downloadResult.architecture,
                    msix = downloadResult.msix,
                    msi = downloadResult.msi,
                    zip = downloadResult.zip,
                    upgradeBehavior = upgradeBehavior,
                    installerSwitches = installerSwitches,
                    msixBundle = downloadResult.msixBundle,
                    gitHubDetection = gitHubDetection,
                    previousManifestData = previousManifestData,
                    onAddInstaller = { installers += it }
                )
            }
            info(additionalInstallerPrompt)
            val loop = currentContext.terminal.yesNoMenu(
                default = installers.size < (previousInstallerManifest?.installers?.size ?: 0)
            ).prompt()
        } while (loop)
        val pageScraper = if (!installerUrl.host.equals(GitHubDetection.gitHubWebsite, true)) {
            WebPageScraper(installerUrl)
        } else {
            null
        }
        fileExtensions = prompt(FileExtensions(installers.size, previousInstallerManifest))
        protocols = prompt(Protocols(installers.size, previousInstallerManifest))
        commands = prompt(Commands(installers.size, previousInstallerManifest))
        installerSuccessCodes = prompt(InstallerSuccessCodes(installers.size, previousInstallerManifest))
        installModes = prompt(InstallModes(installers.size, previousInstallerManifest))
        defaultLocale = prompt(Locale.Package(previousManifestData.defaultLocaleManifest))
        publisher = downloadResult.publisherDisplayName ?: prompt(Publisher(previousManifestData.defaultLocaleManifest))
        packageName = downloadResult.msix?.displayName ?: prompt(PackageName(downloadResult.msi, previousManifestData.defaultLocaleManifest))
        moniker = prompt(Moniker(previousManifestData.defaultLocaleManifest))
        publisherUrl = gitHubDetection?.publisherUrl ?: prompt(Publisher.Url(previousManifestData.defaultLocaleManifest))
        author = prompt(Author(previousManifestData.defaultLocaleManifest))
        packageUrl = gitHubDetection?.packageUrl ?: prompt(PackageUrl(previousManifestData.defaultLocaleManifest))
        license = gitHubDetection?.license ?: prompt(License(previousManifestData.defaultLocaleManifest))
        licenseUrl = gitHubDetection?.licenseUrl ?: prompt(License.Url(previousManifestData.defaultLocaleManifest))
        copyright = prompt(Copyright(previousManifestData.defaultLocaleManifest))
        copyrightUrl = prompt(Copyright.Url(previousManifestData.defaultLocaleManifest))
        tags = gitHubDetection?.topics ?: prompt(Tags(previousManifestData.defaultLocaleManifest))
        shortDescription = if (gitHubDetection?.shortDescription != null && previousManifestData.defaultLocaleManifest?.shortDescription != null) {
            gitHubDetection.shortDescription
        } else {
            prompt(Description.Short(downloadResult.msix))
        }
        description = prompt(Description.Long(previousManifestData.defaultLocaleManifest))
        releaseNotesUrl = gitHubDetection?.releaseNotesUrl ?: prompt(ReleaseNotesUrl)
        val files = Manifest.createFiles(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale,
            license = license ?: previousManifestData.defaultLocaleManifest!!.license,
            licenseUrl = licenseUrl,
            author = author,
            publisherUrl = publisherUrl,
            packageUrl = packageUrl,
            copyright = copyright,
            copyrightUrl = copyrightUrl,
            shortDescription = shortDescription ?: previousManifestData.defaultLocaleManifest!!.license,
            moniker = moniker,
            installers = installers,
            packageName = packageName,
            publisher = publisher,
            manifestOverride = manifestOverride,
            gitHubDetection = gitHubDetection,
            pageScraper = pageScraper,
            previousManifestData = previousManifestData
        )
        for (manifest in files.values.map(Manifest::toString)) {
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
                        previousManifestData = previousManifestData,
                        terminal = currentContext.terminal
                    ).also { success("Pull request created: ${it.htmlUrl}") }
                }
                ManifestResultOption.WriteToFiles -> writeFiles(files, currentContext.terminal)
                else -> return@runBlocking
            }
        }
    }

    companion object {
        private const val additionalInstallerPrompt = "Do you want to create another installer?"
    }
}
