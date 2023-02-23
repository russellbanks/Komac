package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import utils.menu

object InstallerScope : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.installerScopePrompt() {
        if (
            installerManifestData.scope == null &&
            installerManifestData.installerType != InstallerManifest.Installer.InstallerType.PORTABLE
        ) {
            val previousValue = getPreviousValue()
            println(colors.brightYellow(installerScopeInfo))
            previousValue?.let { println(colors.muted("Previous value: $previousValue")) }
            installerManifestData.scope = menu(
                items = InstallerManifest.Installer.Scope.values().toList(),
                default = previousValue
            ).prompt()
            println()
        }
    }

    private suspend fun getPreviousValue(): InstallerManifest.Installer.Scope? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.scope?.toPerScopeInstallerType() ?: it.installers.getOrNull(installerManifestData.installers.size)?.scope
        }
    }

    const val const = "Installer Scope"
    private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
}
