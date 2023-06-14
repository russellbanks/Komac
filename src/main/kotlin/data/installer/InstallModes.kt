package data.installer

import io.menu.prompts.CheckMenuPrompt
import schemas.manifest.InstallerManifest

class InstallModes(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
) : CheckMenuPrompt<InstallerManifest.InstallModes> {
    override val name: String = "Install modes"

    @OptIn(ExperimentalStdlibApi::class)
    override val items: List<InstallerManifest.InstallModes> = InstallerManifest.InstallModes.entries

    override val defaultChecked: List<InstallerManifest.InstallModes>
        get() = previousInstallerManifest?.let { installerManifest ->
            installerManifest.installModes
                ?: installerManifest.installers.getOrNull(currentInstallerIndex)?.installModes
        }.orEmpty()
}
