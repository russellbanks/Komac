package data.installer

import commands.interfaces.MenuPrompt
import data.AllManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object InstallerType : MenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = "Installer type"

    @OptIn(ExperimentalStdlibApi::class)
    override val items = InstallerManifest.InstallerType.entries

    override val default: InstallerManifest.InstallerType?
        get() = PreviousManifestData.installerManifest?.run {
            installerType ?: installers.getOrNull(AllManifestData.installers.size)?.installerType
        }
}
