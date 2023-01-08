import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.VersionManifestData
import data.YamlConfig
import data.installer.Architecture.architecturePrompt
import data.installer.Commands.commandsPrompt
import data.installer.FileExtensions.fileExtensionsPrompt
import data.installer.InstallModes.installModesPrompt
import data.installer.InstallerScope.installerScopePrompt
import data.installer.InstallerSuccessCodes.installerSuccessCodesPrompt
import data.installer.InstallerSwitch.installerSwitchPrompt
import data.installer.InstallerType.installerTypePrompt
import data.installer.ProductCode.productCodePrompt
import data.installer.Protocols.protocolsPrompt
import data.installer.ReleaseDate.releaseDatePrompt
import data.installer.UpgradeBehaviour.upgradeBehaviourPrompt
import data.locale.Author.authorPrompt
import data.locale.Copyright.copyrightPrompt
import data.locale.Description.descriptionPrompt
import data.locale.DescriptionType
import data.locale.License.licensePrompt
import data.locale.LocaleUrl
import data.locale.Moniker.monikerPrompt
import data.locale.Publisher.publisherPrompt
import data.locale.Tags.tagsPrompt
import data.shared.Locale.localePrompt
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageName.packageNamePrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import data.shared.Url.localeUrlPrompt
import input.InstallerSwitch
import input.ManifestResultOption
import input.Polar
import input.PromptType
import input.Prompts
import input.Prompts.pullRequestPrompt
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.LocaleManifest
import schemas.SchemasImpl
import schemas.TerminalInstance

class NewManifest : CliktCommand(name = "new"), KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocalManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private var previousManifestData: PreviousManifestData? = null

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt()
            launch { if (!sharedManifestData.isNewPackage) previousManifestData = get() }
            launch {
                packageVersionPrompt()
                do {
                    installerDownloadPrompt()
                    architecturePrompt()
                    installerTypePrompt()
                    InstallerSwitch.values().forEach { installerSwitchPrompt(it) }
                    localePrompt(PromptType.InstallerLocale)
                    productCodePrompt()
                    installerScopePrompt()
                    upgradeBehaviourPrompt()
                    releaseDatePrompt()
                    val installer = installerManifestData.createInstaller()
                    addMsixBundlePackages(installer)
                    installerManifestData.installers += installer
                    val shouldContinue = shouldLoopPrompt()
                } while (shouldContinue)
                fileExtensionsPrompt()
                protocolsPrompt()
                commandsPrompt()
                installerSuccessCodesPrompt()
                installModesPrompt()
                localePrompt(PromptType.PackageLocale)
                publisherPrompt()
                packageNamePrompt()
                monikerPrompt()
                localeUrlPrompt(LocaleUrl.PublisherUrl)
                localeUrlPrompt(LocaleUrl.PublisherSupportUrl)
                localeUrlPrompt(LocaleUrl.PublisherPrivacyUrl)
                authorPrompt()
                localeUrlPrompt(LocaleUrl.PackageUrl)
                licensePrompt()
                localeUrlPrompt(LocaleUrl.LicenseUrl)
                copyrightPrompt()
                localeUrlPrompt(LocaleUrl.CopyrightUrl)
                tagsPrompt()
                DescriptionType.values().forEach { descriptionPrompt(it) }
                localeUrlPrompt(LocaleUrl.ReleaseNotesUrl)
                pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> commitAndPullRequest()
                        ManifestResultOption.WriteToFiles -> println(brightWhite("Writing files"))
                        else -> println(brightWhite("Exiting"))
                    }
                }
            }
        }
    }

    private fun addMsixBundlePackages(installer: InstallerManifest.Installer) {
        sharedManifestData.msixBundle?.let { msixBundle ->
            msixBundle.packages?.forEachIndexed { index, individualPackage ->
                if (index == 0) return@forEachIndexed
                individualPackage.processorArchitecture?.let { architecture ->
                    installerManifestData.installers += installer.copy(
                        architecture = architecture,
                        platform = individualPackage.targetDeviceFamily?.map {
                            it.toPerInstallerPlatform()
                        },
                    )
                }
            }
            sharedManifestData.msixBundle = null
        }
    }

    private suspend fun commitAndPullRequest() {
        previousManifestData?.apply {
            remoteVersionDataJob.join()
            remoteLocaleDataJob.join()
            remoteDefaultLocaleDataJob.join()
        }
        val githubImpl = get<GitHubImpl>()
        val repository = githubImpl.getWingetPkgsFork() ?: return
        val ref = githubImpl.createBranchFromDefaultBranch(repository) ?: return
        githubImpl.commitFiles(
            repository = repository,
            branch = ref,
            files = listOf(
                githubImpl.installerManifestGitHubPath to installerManifestData.createInstallerManifest(),
                githubImpl.defaultLocaleManifestGitHubPath to defaultLocalManifestData.createDefaultLocaleManifest(),
                githubImpl.versionManifestGitHubPath to versionManifestData.createVersionManifest(),
            ) + previousManifestData?.remoteLocaleData?.map { localeManifest ->
                githubImpl.getLocaleManifestGitHubPath(localeManifest.packageLocale) to localeManifest.copy(
                    packageIdentifier = sharedManifestData.packageIdentifier,
                    packageVersion = sharedManifestData.packageVersion,
                    manifestVersion = "1.4.0"
                ).let {
                    githubImpl.buildManifestString(get<SchemasImpl>().localeSchema.id) {
                        appendLine(YamlConfig.default.encodeToString(LocaleManifest.serializer(), it))
                    }
                }
            }.orEmpty()
        )
    }

    private fun Terminal.shouldLoopPrompt(): Boolean {
        var promptInput: Char?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(additionalInstallerInfo))
                    Polar.values().forEach {
                        val textColour = if (it == Polar.No) brightGreen else brightWhite
                        cell(textColour("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                    }
                }
            )
            promptInput = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = when {
                    (previousManifestData?.remoteInstallerData?.installers?.size ?: 0) >
                        installerManifestData.installers.size -> Polar.Yes.name.first().toString()
                    else -> Polar.No.name.first().toString()
                },
            )?.trim()?.lowercase()?.firstOrNull()
            println()
        } while (
            promptInput != Polar.Yes.toString().first().lowercaseChar() &&
            promptInput != Polar.No.toString().first().lowercaseChar()
        )
        return promptInput == Polar.Yes.toString().first().lowercaseChar()
    }

    companion object {
        private const val additionalInstallerInfo = "Do you want to create another installer?"
    }
}
