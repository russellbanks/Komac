package data.installer

import commands.interfaces.RadioMenuPrompt
import data.AllManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest
import utils.extension

object InstallerType : RadioMenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = "Installer type"

    @OptIn(ExperimentalStdlibApi::class)
    override val items = if (AllManifestData.installerUrl.extension.equals("exe", ignoreCase = true)) {
        listOf(
            InstallerManifest.InstallerType.BURN,
            InstallerManifest.InstallerType.EXE,
            InstallerManifest.InstallerType.PORTABLE
        )
    } else {
        InstallerManifest.InstallerType.entries
    }

    override val default: InstallerManifest.InstallerType? get() = PreviousManifestData.installerManifest?.run {
        installerType ?: installers.getOrNull(AllManifestData.installers.size)?.installerType
    }
}
