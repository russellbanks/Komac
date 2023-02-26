package data.installer

import Errors
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.PreviousManifestData
import input.InstallerSwitch
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest

object InstallerSwitch : KoinComponent {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.installerSwitchPrompt(installerSwitch: InstallerSwitch) = with(allManifestData) {
        if (
            installerType == InstallerManifest.Installer.InstallerType.EXE ||
            installerSwitch == InstallerSwitch.Custom
        ) {
            val isRequired = installerType == InstallerManifest.Installer.InstallerType.EXE &&
                installerSwitch != InstallerSwitch.Custom
            switchInfo(installerType, installerSwitch).also { (info, infoColor) ->
                println(infoColor(info))
            }
            info(switchExample(installerSwitch))
            installerSwitches[installerSwitch] = prompt(
                prompt = installerSwitch.toString(),
                default = getPreviousValue(installerSwitch)?.also { muted("Previous $installerSwitch: $it") },
                convert = { input ->
                    isInstallerSwitchValid(switch = input, installerSwitch = installerSwitch, canBeBlank = !isRequired)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input)
                }
            )?.takeIf { it.isNotBlank() }?.trim()
            println()
        }
    }

    private fun isInstallerSwitchValid(
        switch: String,
        installerSwitch: InstallerSwitch,
        canBeBlank: Boolean = false
    ): String? {
        val (minBoundary, maxBoundary) = 1 to if (installerSwitch == InstallerSwitch.Custom) 2048 else 512
        return when {
            switch.isBlank() && !canBeBlank -> Errors.blankInput(installerSwitch.toString())
            switch.length > maxBoundary -> Errors.invalidLength(min = minBoundary, max = maxBoundary)
            else -> null
        }
    }

    private suspend fun getPreviousValue(installerSwitch: InstallerSwitch): String? {
        return previousManifestData.remoteInstallerData.await()?.let {
            when (installerSwitch) {
                InstallerSwitch.Silent -> {
                    it.installerSwitches?.silent
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches?.silent
                }
                InstallerSwitch.SilentWithProgress -> {
                    it.installerSwitches?.silentWithProgress
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches?.silentWithProgress
                }
                InstallerSwitch.Custom -> {
                    it.installerSwitches?.custom
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches?.custom
                }
            }
        }
    }

    private suspend fun switchInfo(
        installerType: InstallerManifest.Installer.InstallerType?,
        installerSwitch: InstallerSwitch
    ): Pair<String, TextColors> = with(allManifestData) {
        val isRequired = installerType == InstallerManifest.Installer.InstallerType.EXE &&
            installerSwitch != InstallerSwitch.Custom
        return buildString {
            append(
                when {
                    installerType == InstallerManifest.Installer.InstallerType.EXE &&
                        installerSwitch != InstallerSwitch.Custom -> Prompts.required
                    else -> Prompts.optional
                }
            )
            append(" Enter the ${installerSwitch.toString().lowercase()} install switch")
        } to if (getPreviousValue(installerSwitch).isNullOrBlank() && isRequired) brightGreen else brightYellow
    }

    private fun switchExample(installerSwitch: InstallerSwitch): String {
        return buildString {
            append("Example: ")
            append(
                when (installerSwitch) {
                    InstallerSwitch.Silent -> "/S, -verysilent, /qn, --silent, /exenoui."
                    InstallerSwitch.SilentWithProgress -> "/S, -silent, /qb, /exebasicui."
                    InstallerSwitch.Custom -> "/norestart, -norestart"
                }
            )
        }
    }
}
