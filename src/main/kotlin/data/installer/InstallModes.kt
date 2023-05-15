package data.installer

import commands.interfaces.CheckMenuPrompt
import data.ManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object InstallModes : CheckMenuPrompt<InstallerManifest.InstallModes> {
    override val name: String = "Install modes"

    @OptIn(ExperimentalStdlibApi::class)
    override val items: List<InstallerManifest.InstallModes> = InstallerManifest.InstallModes.entries

    override val defaultChecked: List<InstallerManifest.InstallModes>? get() = PreviousManifestData.installerManifest
        ?.let { installerManifest ->
            installerManifest.installModes
                ?: installerManifest.installers.getOrNull(ManifestData.installers.size)?.installModes
        }
}
