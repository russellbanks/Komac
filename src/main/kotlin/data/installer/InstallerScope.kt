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
    fun installerScopePrompt(terminal: Terminal) = with(allManifestData) {
        with(terminal) {
            if (scope == null && installerType != InstallerManifest.Installer.InstallerType.PORTABLE) {
                val previousValue = getPreviousValue()
                println(colors.brightYellow(installerScopeInfo))
                previousValue?.let { println(colors.muted("Previous value: $it")) }
                scope = menu(
                    items = InstallerManifest.Installer.Scope.values().toList(),
                    default = previousValue,
                    optionalItemName = "No idea"
                ).prompt()
                println()
            }
        }
    }

    private fun getPreviousValue(): InstallerManifest.Installer.Scope? = with(allManifestData) {
        return previousInstallerManifest?.let {
            it.scope?.toPerScopeInstallerType() ?: it.installers.getOrNull(installers.size)?.scope
        }
    }

    companion object {
        const val const = "Installer Scope"
        private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
    }
}
