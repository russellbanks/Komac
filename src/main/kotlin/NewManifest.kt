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
import input.Polar
import input.PromptType
import input.Prompts
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.LocaleManifest
import schemas.SchemasImpl
import schemas.TerminalInstance

class NewManifest : CliktCommand(name = "new"), KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocalManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var previousManifestData: PreviousManifestData

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
                    installerManifestData.addInstaller()
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
                println(
                    verticalLayout {
                        cell(
                            brightYellow(
                                "Would you like to make a pull request to add " +
                                    "${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}?"
                            )
                        )
                        Polar.values().forEach {
                            cell(brightWhite("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                        }
                    }
                )
                prompt(
                    prompt = brightWhite(Prompts.enterChoice),
                    showChoices = false,
                    choices = Polar.values().map { it.name.first().toString() },
                )?.trim()?.firstOrNull().also { if (it == Polar.Yes.toString().first()) commitAndPullRequest() }
            }
        }
    }

    private suspend fun commitAndPullRequest() {
        previousManifestData.remoteVersionDataJob.join()
        previousManifestData.remoteLocaleDataJob.join()
        previousManifestData.remoteDefaultLocaleDataJob.join()
        val githubImpl = get<GitHubImpl>()
        val repository = githubImpl.getWingetPkgsFork() ?: return
        val ref = githubImpl.createBranchFromDefaultBranch(repository) ?: return
        githubImpl.commitFiles(
            repository = repository,
            branch = ref,
            files = listOf(
                githubImpl.installerManifestGitHubPath to installerManifestData.createInstallerManifest(),
                githubImpl.defaultLocaleManifestGitHubPath to
                    defaultLocalManifestData.createDefaultLocaleManifest(),
                githubImpl.versionManifestGitHubPath to versionManifestData.createVersionManifest(),
            ) + previousManifestData.remoteLocaleData?.map { localeManifest ->
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
                    (previousManifestData.remoteInstallerData?.installers?.size ?: 0) >
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
