package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.PreviousManifestData
import input.Prompts
import schemas.manifest.InstallerManifest
import utils.menu

object InstallerScope {
    const val const = "Installer Scope"
    private const val installerScopeInfo = "${Prompts.optional} Enter the $const"

    @OptIn(ExperimentalStdlibApi::class)
    fun installerScopePrompt(terminal: Terminal) = with(terminal) {
        if (AllManifestData.scope == null && AllManifestData.installerType != InstallerManifest.InstallerType.PORTABLE) {
            val previousValue = previousValue
            println(colors.brightYellow(installerScopeInfo))
            previousValue?.let { println(colors.muted("Previous value: $it")) }
            AllManifestData.scope = menu<InstallerManifest.Scope> {
                items = InstallerManifest.Scope.entries
                default = previousValue
                optionalItemName = "No idea"
            }.prompt()
            println()
        }
    }

    private val previousValue: InstallerManifest.Scope?
        get() = PreviousManifestData.installerManifest?.let {
            it.scope ?: it.installers.getOrNull(AllManifestData.installers.size)?.scope
        }
}
