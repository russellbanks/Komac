package data.installer

import commands.interfaces.MenuPrompt
import schemas.manifest.InstallerManifest

class InstallerType(
    private val previousInstaller: InstallerManifest?,
    private val installersSize: Int
) : MenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = "Installer type"

    @OptIn(ExperimentalStdlibApi::class)
    override val items = InstallerManifest.InstallerType.entries

    override val default = getPreviousValue()

    private fun getPreviousValue(): InstallerManifest.InstallerType? {
        return previousInstaller?.run { installerType ?: installers.getOrNull(installersSize)?.installerType }
    }
}
