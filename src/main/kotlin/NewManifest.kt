import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.InstallerManifestData
import data.VersionManifestData
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
import data.shared.Locale.installerLocalePrompt
import data.shared.Locale.packageLocalePrompt
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageName.packageNamePrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import data.shared.Url.localeUrlPrompt
import input.InstallerSwitch
import input.Polar
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocalManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()

    suspend fun main() {
        with(terminal) {
            packageIdentifierPrompt()
            packageVersionPrompt()
            do {
                installerDownloadPrompt()
                architecturePrompt()
                installerTypePrompt()
                InstallerSwitch.values().forEach { installerSwitchPrompt(it) }
                installerLocalePrompt()
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
            packageLocalePrompt()
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
            installerManifestData.createInstallerManifest()
            println()
            defaultLocalManifestData.createDefaultLocaleManifest()
            println()
            versionManifestData.createVersionManifest()
        }
    }

    private fun Terminal.shouldLoopPrompt(): Boolean {
        var promptInput: Char?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(additionalInstallerInfo))
                    Polar.values().forEach {
                        val textColour = if (it == Polar.No) brightGreen else brightWhite
                        cell(
                            textColour(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${it.name.first()}] ")
                                    append(it.name)
                                }
                            )
                        )
                    }
                }
            )
            promptInput = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = Polar.No.toString().first().toString()
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
