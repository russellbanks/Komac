package data.installer

import commands.interfaces.RadioMenuPrompt
import data.ManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object InstallerScope : RadioMenuPrompt<InstallerManifest.Scope> {
    override val name: String = "Installer scope"

    @OptIn(ExperimentalStdlibApi::class)
    override val items: List<InstallerManifest.Scope> = InstallerManifest.Scope.entries

    override val default: InstallerManifest.Scope? get() = PreviousManifestData.installerManifest?.let {
        it.scope ?: it.installers.getOrNull(ManifestData.installers.size)?.scope
    }
}
