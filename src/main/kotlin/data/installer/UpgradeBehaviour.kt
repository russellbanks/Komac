package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import input.Prompts
import schemas.manifest.InstallerManifest
import utils.menu

class UpgradeBehaviour(
    private val allManifestData: AllManifestData,
    private val previousInstallerManifest: InstallerManifest?
) : CommandPrompt<InstallerManifest.Installer.UpgradeBehavior?> {
    override fun prompt(terminal: Terminal): InstallerManifest.Installer.UpgradeBehavior? = with(terminal) {
        if (allManifestData.installerType == InstallerManifest.Installer.InstallerType.PORTABLE) return null
        val previousValue = getPreviousValue()
        println(colors.brightYellow(upgradeBehaviourInfo))
        previousValue?.let { println(colors.muted("Previous value: $previousValue")) }
        return menu(
            items = InstallerManifest.Installer.UpgradeBehavior.values().toList(),
            default = previousValue ?: InstallerManifest.Installer.UpgradeBehavior.Install
        ).prompt()
    }

    override fun getError(input: String?): String? = null

    private fun getPreviousValue(): InstallerManifest.Installer.UpgradeBehavior? = with(allManifestData) {
        return previousInstallerManifest?.let {
            it.upgradeBehavior?.toPerInstallerUpgradeBehaviour()
                ?: it.installers.getOrNull(installers.size)?.upgradeBehavior
        }
    }

    companion object {
        private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
    }
}
