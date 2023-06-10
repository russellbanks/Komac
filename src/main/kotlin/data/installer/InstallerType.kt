package data.installer

import io.menu.prompts.RadioMenuPrompt
import data.ManifestData
import data.PreviousManifestData
import schemas.manifest.InstallerManifest
import utils.extension

object InstallerType : RadioMenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = "Installer type"

    @OptIn(ExperimentalStdlibApi::class)
    override val items get() = if (ManifestData.installerUrl.extension.equals("exe", ignoreCase = true)) {
        listOf(
            InstallerManifest.InstallerType.BURN,
            InstallerManifest.InstallerType.EXE,
            InstallerManifest.InstallerType.PORTABLE
        )
    } else {
        InstallerManifest.InstallerType.entries
    }

    override val default: InstallerManifest.InstallerType? get() = PreviousManifestData.installerManifest?.run {
        installerType ?: installers.getOrNull(ManifestData.installers.size)?.installerType
    }
}
