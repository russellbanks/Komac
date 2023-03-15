package data.installer

import commands.interfaces.MenuPrompt
import data.AllManifestData
import schemas.manifest.InstallerManifest

class UpgradeBehaviour(
    private val allManifestData: AllManifestData,
    private val previousInstallerManifest: InstallerManifest?,
) : MenuPrompt<InstallerManifest.Installer.UpgradeBehavior?> {
    override val name: String = "Upgrade behaviour"

    override val default: InstallerManifest.Installer.UpgradeBehavior =
        getPreviousValue() ?: InstallerManifest.Installer.UpgradeBehavior.Install

    override val items: List<InstallerManifest.Installer.UpgradeBehavior?> =
        InstallerManifest.Installer.UpgradeBehavior.values().toList()

    private fun getPreviousValue(): InstallerManifest.Installer.UpgradeBehavior? = with(allManifestData) {
        return previousInstallerManifest?.let {
            it.upgradeBehavior?.toPerInstallerUpgradeBehaviour()
                ?: it.installers.getOrNull(installers.size)?.upgradeBehavior
        }
    }
}
