package data.installer

import commands.interfaces.RadioMenuPrompt
import data.ManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest

object InstallerType : RadioMenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = "Installer type"

    @OptIn(ExperimentalStdlibApi::class)
    override val items = InstallerManifest.InstallerType.entries

    override val default: InstallerManifest.InstallerType? get() = PreviousManifestData.installerManifest?.run {
        installerType ?: installers.getOrNull(ManifestData.installers.size)?.installerType
    }
}
