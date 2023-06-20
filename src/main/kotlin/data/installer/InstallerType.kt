package data.installer

import io.ktor.http.Url
import io.menu.prompts.RadioMenuPrompt
import schemas.manifest.InstallerManifest
import utils.extension

class InstallerType(
    private val installerUrl: Url,
    private val allInstallers: List<InstallerManifest.Installer>,
    private val previousInstallerManifest: InstallerManifest?
) : RadioMenuPrompt<InstallerManifest.InstallerType> {
    override val name: String = InstallerType.name

    @OptIn(ExperimentalStdlibApi::class)
    override val items get() = if (installerUrl.extension.equals("exe", ignoreCase = true)) {
        listOf(
            InstallerManifest.InstallerType.BURN,
            InstallerManifest.InstallerType.EXE,
            InstallerManifest.InstallerType.PORTABLE
        )
    } else {
        InstallerManifest.InstallerType.entries
    }

    override val default: InstallerManifest.InstallerType? get() = previousInstallerManifest?.run {
        installerType ?: installers.getOrNull(allInstallers.size)?.installerType
    }

    companion object {
        const val name = "Installer type"
    }
}
