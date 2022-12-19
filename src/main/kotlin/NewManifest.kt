import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.Architecture.architecturePrompt
import data.Commands.commandsPrompt
import data.FileExtensions.fileExtensionsPrompt
import data.InstallModes.installModesPrompt
import data.InstallerLocale.installerLocalePrompt
import data.InstallerManifestData
import data.InstallerScope.installerScopePrompt
import data.InstallerSuccessCodes.installerSuccessCodesPrompt
import data.InstallerSwitch.installerSwitchPrompt
import data.InstallerType.installerTypePrompt
import data.InstallerUrl.installerDownloadPrompt
import data.PackageIdentifier.packageIdentifierPrompt
import data.PackageVersion.packageVersionPrompt
import data.ProductCode.productCodePrompt
import data.Protocols.protocolsPrompt
import data.ReleaseDate.releaseDatePrompt
import data.UpgradeBehaviour.upgradeBehaviourPrompt
import input.InstallerSwitch
import input.Polar
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class NewManifest(private val terminal: Terminal) : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()

    suspend fun main() {
        with(terminal) {
            packageIdentifierPrompt()
            packageVersionPrompt()
            do {
                installerDownloadPrompt()
                architecturePrompt()
                installerTypePrompt()
                installerSwitchPrompt(InstallerSwitch.Silent)
                installerSwitchPrompt(InstallerSwitch.SilentWithProgress)
                installerSwitchPrompt(InstallerSwitch.Custom)
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
            installerManifestData.createInstallerManifest()
        }
    }

    private fun Terminal.shouldLoopPrompt(): Boolean {
        var promptInput: Char?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.additionalInstallerInfo))
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
}
