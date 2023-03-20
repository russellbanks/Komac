package data.installer

import Errors
import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import extensions.YamlExtensions.convertToList
import schemas.manifest.InstallerManifest

class InstallModes(
    previousInstallerManifest: InstallerManifest?,
    private val installerSize: Int
) : ListPrompt<InstallerManifest.InstallModes> {
    override val name: String = "Install modes"

    override val description: String = "List of supported installer modes"

    override val extraText: String = "Options: ${InstallerManifest.InstallModes.values().joinToString()}"

    override val default: List<InstallerManifest.InstallModes>? = previousInstallerManifest?.let { installerManifest ->
        installerManifest.installModes
            ?: installerManifest.installers
                .getOrNull(installerSize)
                ?.installModes
                ?.map(InstallerManifest.Installer.InstallModes::toManifestInstallMode)
    }

    override val validationRules: ListValidationRules<InstallerManifest.InstallModes> = ListValidationRules(
        maxItems = 3,
        transform = { convertToList(it).toInstallModes() }
    ) { installModes ->
        if (installModes.any { it !in InstallerManifest.InstallModes.values() }) {
            Errors.invalidEnum(InstallerManifest.InstallModes.values().map(InstallerManifest.InstallModes::toString))
        } else {
            null
        }
    }

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes> {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }
    }
}
