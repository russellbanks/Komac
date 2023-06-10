package data.installer

import io.menu.prompts.RadioMenuPrompt
import data.ManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object UpgradeBehaviour : RadioMenuPrompt<InstallerManifest.UpgradeBehavior?> {
    override val name: String = "Upgrade behaviour"

    override val default: InstallerManifest.UpgradeBehavior = previousValue ?: InstallerManifest.UpgradeBehavior.Install

    @OptIn(ExperimentalStdlibApi::class)
    override val items = InstallerManifest.UpgradeBehavior.entries

    private val previousValue: InstallerManifest.UpgradeBehavior?
        get() = PreviousManifestData.installerManifest?.let {
            it.upgradeBehavior ?: it.installers.getOrNull(ManifestData.installers.size)?.upgradeBehavior
        }
}
