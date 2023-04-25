package data.installer

import commands.interfaces.MenuPrompt
import data.AllManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object InstallerScope : MenuPrompt<InstallerManifest.Scope> {
    override val name: String = "Installer scope"

    @OptIn(ExperimentalStdlibApi::class)
    override val items: List<InstallerManifest.Scope> = InstallerManifest.Scope.entries

    override val default: InstallerManifest.Scope?
        get() = PreviousManifestData.installerManifest?.let {
            it.scope ?: it.installers.getOrNull(AllManifestData.installers.size)?.scope
        }
}
