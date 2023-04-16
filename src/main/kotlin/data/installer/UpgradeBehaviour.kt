package data.installer

import commands.interfaces.MenuPrompt
import data.AllManifestData
import schemas.manifest.InstallerManifest

class UpgradeBehaviour(
    private val allManifestData: AllManifestData,
    private val previousInstallerManifest: InstallerManifest?,
) : MenuPrompt<InstallerManifest.UpgradeBehavior?> {
    override val name: String = "Upgrade behaviour"

    override val default: InstallerManifest.UpgradeBehavior =
        getPreviousValue() ?: InstallerManifest.UpgradeBehavior.Install

    @OptIn(ExperimentalStdlibApi::class)
    override val items = InstallerManifest.UpgradeBehavior.entries

    private fun getPreviousValue(): InstallerManifest.UpgradeBehavior? = with(allManifestData) {
        return previousInstallerManifest?.let {
            it.upgradeBehavior ?: it.installers.getOrNull(installers.size)?.upgradeBehavior
        }
    }
}
