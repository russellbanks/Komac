import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.Architecture.architecturePrompt
import data.Commands.commandsPrompt
import data.DefaultLocaleManifestData
import data.FileExtensions.fileExtensionsPrompt
import data.InstallModes.installModesPrompt
import data.InstallerManifestData
import data.InstallerScope.installerScopePrompt
import data.InstallerSuccessCodes.installerSuccessCodesPrompt
import data.InstallerSwitch.installerSwitchPrompt
import data.InstallerType.installerTypePrompt
import data.InstallerUrl.installerDownloadPrompt
import data.Locale.installerLocalePrompt
import data.Locale.packageLocalePrompt
import data.Moniker.monikerPrompt
import data.PackageIdentifier.packageIdentifierPrompt
import data.PackageName.packageNamePrompt
import data.PackageVersion.packageVersionPrompt
import data.ProductCode.productCodePrompt
import data.Protocols.protocolsPrompt
import data.Publisher.publisherPrompt
import data.ReleaseDate.releaseDatePrompt
import data.UpgradeBehaviour.upgradeBehaviourPrompt
import data.VersionManifestData
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
