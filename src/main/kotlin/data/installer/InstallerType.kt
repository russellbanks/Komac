package data.installer

import commands.interfaces.MenuPrompt
import schemas.manifest.InstallerManifest

class InstallerType(
    private val previousInstaller: InstallerManifest?,
    private val installersSize: Int
) : MenuPrompt<InstallerManifest.Installer.InstallerType> {
    override val name: String = "Installer type"

    override val items: List<InstallerManifest.Installer.InstallerType> =
        InstallerManifest.Installer.InstallerType.values().toList()

    override val default: InstallerManifest.Installer.InstallerType? = getPreviousValue()

    private fun getPreviousValue(): InstallerManifest.Installer.InstallerType? {
        return previousInstaller?.run {
            installerType?.toPerInstallerType() ?: installers.getOrNull(installersSize)?.installerType
        }
    }
}
