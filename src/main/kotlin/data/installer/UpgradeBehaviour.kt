package data.installer

import io.menu.prompts.RadioMenuPrompt
import schemas.manifest.InstallerManifest

class UpgradeBehaviour(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
) : RadioMenuPrompt<InstallerManifest.UpgradeBehavior?> {
    override val name: String = "Upgrade behaviour"

    override val default: InstallerManifest.UpgradeBehavior = previousValue ?: InstallerManifest.UpgradeBehavior.Install

    override val items = InstallerManifest.UpgradeBehavior.entries

    private val previousValue: InstallerManifest.UpgradeBehavior?
        get() = previousInstallerManifest?.let {
            it.upgradeBehavior ?: it.installers.getOrNull(currentInstallerIndex)?.upgradeBehavior
        }
}
