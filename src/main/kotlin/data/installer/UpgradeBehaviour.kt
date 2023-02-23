package data.installer

import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import utils.menu

object UpgradeBehaviour : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.upgradeBehaviourPrompt() {
        if (installerManifestData.installerType == InstallerManifest.Installer.InstallerType.PORTABLE) return
        val previousValue = getPreviousValue()
        println(colors.brightYellow(upgradeBehaviourInfo))
        previousValue?.let { println(colors.muted("Previous value: $previousValue")) }
        installerManifestData.upgradeBehavior = menu(
            items = InstallerManifest.Installer.UpgradeBehavior.values().toList(),
            default = previousValue
        ).prompt()
        println()
    }

    private suspend fun getPreviousValue(): InstallerManifest.Installer.UpgradeBehavior? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.upgradeBehavior?.toPerInstallerUpgradeBehaviour()
                ?: it.installers.getOrNull(installerManifestData.installers.size)?.upgradeBehavior
        }
    }

    private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
}
