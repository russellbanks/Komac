package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import utils.menu

object InstallerScope : KoinComponent {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.installerScopePrompt() = with(allManifestData) {
        if (scope == null && installerType != InstallerManifest.Installer.InstallerType.PORTABLE) {
            val previousValue = getPreviousValue()
            println(colors.brightYellow(installerScopeInfo))
            previousValue?.let { println(colors.muted("Previous value: $previousValue")) }
            scope = menu(
                items = InstallerManifest.Installer.Scope.values().toList(),
                default = previousValue,
                optionalItemName = "No idea"
            ).prompt()
            println()
        }
    }

    private suspend fun getPreviousValue(): InstallerManifest.Installer.Scope? = with(allManifestData) {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.scope?.toPerScopeInstallerType() ?: it.installers.getOrNull(installers.size)?.scope
        }
    }

    const val const = "Installer Scope"
    private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
}
