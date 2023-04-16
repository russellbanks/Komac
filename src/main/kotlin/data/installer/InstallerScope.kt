package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import input.Prompts
import schemas.manifest.InstallerManifest
import utils.menu

class InstallerScope(
    private val allManifestData: AllManifestData,
    private val previousInstallerManifest: InstallerManifest?
) {
    @OptIn(ExperimentalStdlibApi::class)
    fun installerScopePrompt(terminal: Terminal) = with(terminal) {
        if (allManifestData.scope == null && allManifestData.installerType != InstallerManifest.InstallerType.PORTABLE) {
            val previousValue = getPreviousValue()
            println(colors.brightYellow(installerScopeInfo))
            previousValue?.let { println(colors.muted("Previous value: $it")) }
            allManifestData.scope = menu(
                items = InstallerManifest.Scope.entries,
                default = previousValue,
                optionalItemName = "No idea"
            ).prompt()
            println()
        }
    }

    private fun getPreviousValue(): InstallerManifest.Scope? = with(allManifestData) {
        return previousInstallerManifest?.let { it.scope ?: it.installers.getOrNull(installers.size)?.scope }
    }

    companion object {
        const val const = "Installer Scope"
        private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
    }
}
