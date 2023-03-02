package commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.validate
import commands.CommandUtils.prompt
import data.AllManifestData
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.ManifestUtils
import data.PreviousManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.installer.Commands
import data.installer.FileExtensions
import data.installer.InstallModes
import data.installer.InstallerScope.installerScopePrompt
import data.installer.InstallerSuccessCodes
import data.installer.InstallerSwitch.installerSwitchPrompt
import data.installer.InstallerType
import data.installer.Protocols
import data.installer.UpgradeBehaviour.upgradeBehaviourPrompt
import data.locale.Author
import data.locale.Copyright
import data.locale.Description.descriptionPrompt
import data.locale.DescriptionType
import data.locale.License
import data.locale.Moniker
import data.locale.PackageUrl
import data.locale.ReleaseNotesUrl
import data.locale.Tags
import data.shared.Locale
import data.shared.PackageIdentifier
import data.shared.PackageIdentifier.getLatestVersion
import data.shared.PackageName
import data.shared.PackageVersion
import data.shared.Publisher
import data.shared.Url.installerDownloadPrompt
import input.ExitCode
import input.FileWriter.writeFiles
import input.InstallerSwitch
import input.ManifestResultOption
import input.Prompts.pullRequestPrompt
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.manifest.EncodeConfig
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import kotlin.system.exitProcess

class NewManifest : CliktCommand(name = "new"), KoinComponent {
    private val tokenStore: TokenStore by inject()
    private val allManifestData: AllManifestData by inject()
    private var previousManifestData: PreviousManifestData? = null
    private val githubImpl: GitHubImpl by inject()
    private val manifestVersion: String? by option().validate {
        require(Regex("^\\d+\\.\\d+\\.\\d+$").matches(it)) { "Manifest version must be in the format X.X.X" }
    }

    override fun run(): Unit = runBlocking {
        manifestVersion?.let { get<Schemas>().manifestOverride = it }
        with(currentContext.terminal) {
            with(allManifestData) {
                if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
                packageIdentifier = prompt(PackageIdentifier)
                if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(currentContext.terminal)
                latestVersion = getLatestVersion(packageIdentifier)
                launch { if (updateState != VersionUpdateState.NewPackage) previousManifestData = get() }
                launch {
                    packageVersion = prompt(PackageVersion)
                    githubImpl.promptIfPullRequestExists(
                        identifier = packageIdentifier,
                        version = packageVersion,
                        terminal = currentContext.terminal
                    )
                    PackageVersion.setUpgradeState(allManifestData)
                    do {
                        installerDownloadPrompt()
                        installerType = installerType ?: prompt(InstallerType)
                        InstallerSwitch.values().forEach { installerSwitchPrompt(it) }
                        installerLocale = prompt(Locale.Installer)
                        installerScopePrompt()
                        upgradeBehaviourPrompt()
                        if (!skipAddInstaller) InstallerManifestData.addInstaller() else skipAddInstaller = false
                        val loop = confirm(colors.info(additionalInstallerInfo)) ?: exitProcess(ExitCode.CtrlC.code)
                    } while (loop)
                    fileExtensions = prompt(FileExtensions)
                    protocols = prompt(Protocols)
                    commands = prompt(Commands)
                    installerSuccessCodes = prompt(InstallerSuccessCodes)
                    installModes = prompt(InstallModes)
                    defaultLocale = prompt(Locale.Package)
                    publisher = prompt(Publisher)
                    packageName = prompt(PackageName)
                    moniker = prompt(Moniker)
                    publisherUrl = prompt(Publisher.Url)
                    author = prompt(Author)
                    packageUrl = prompt(PackageUrl)
                    license = prompt(License)
                    licenseUrl = prompt(License.Url)
                    copyright = prompt(Copyright)
                    copyrightUrl = prompt(Copyright.Url)
                    tags = prompt(Tags)
                    DescriptionType.values().forEach { descriptionPrompt(it) }
                    releaseNotesUrl = prompt(ReleaseNotesUrl)
                    val files = createFiles()
                    files.forEach { manifest ->
                        ManifestUtils.formattedManifestLinesFlow(manifest.second, colors).collect(::echo)
                    }
                    pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                        when (manifestResultOption) {
                            ManifestResultOption.PullRequest -> {
                                githubImpl.commitAndPullRequest(files, currentContext.terminal)
                            }
                            ManifestResultOption.WriteToFiles -> writeFiles(files, currentContext.terminal)
                            else -> return@also
                        }
                    }
                }
            }
        }
    }

    private suspend fun createFiles(): List<Pair<String, String>> = with(allManifestData) {
        return listOf(
            githubImpl.installerManifestName to InstallerManifestData.createInstallerManifest(),
            githubImpl.getDefaultLocaleManifestName() to DefaultLocaleManifestData.createDefaultLocaleManifest(),
            githubImpl.versionManifestName to VersionManifestData.createVersionManifest()
        ) + previousManifestData?.remoteLocaleData?.await()?.map { localeManifest ->
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                manifestVersion = get<Schemas>().manifestOverride ?: Schemas.manifestVersion
            ).let {
                Schemas().buildManifestString(
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
