package data.installer

import io.menu.prompts.RadioMenuPrompt
import schemas.manifest.InstallerManifest

class InstallerScope(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
) : RadioMenuPrompt<InstallerManifest.Scope> {
    override val name: String = InstallerScope.name

    @OptIn(ExperimentalStdlibApi::class)
    override val items: List<InstallerManifest.Scope> = InstallerManifest.Scope.entries

    override val default: InstallerManifest.Scope? get() = previousInstallerManifest?.let {
        it.scope ?: it.installers.getOrNull(currentInstallerIndex)?.scope
    }

    companion object {
        const val name: String = "Installer scope"
    }
}
