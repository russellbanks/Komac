package data.installer

import Errors
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import input.Prompts
import input.Switch
import schemas.manifest.InstallerManifest

class InstallerSwitch(
    private val allManifestData: AllManifestData,
    private val previousInstallerData: InstallerManifest?
) {
    fun installerSwitchPrompt(installerSwitch: Switch, terminal: Terminal) = with(allManifestData) {
        if (
            installerType == InstallerManifest.Installer.InstallerType.EXE ||
            installerSwitch == Switch.Custom
        ) {
            val isRequired = installerType == InstallerManifest.Installer.InstallerType.EXE &&
                installerSwitch != Switch.Custom
            switchInfo(installerType, installerSwitch).also { (info, infoColor) ->
                println(infoColor(info))
            }
            terminal.info(switchExample(installerSwitch))
            installerSwitches
            installerSwitches[installerSwitch] = terminal.prompt(
                prompt = installerSwitch.toString(),
                default = getPreviousValue(installerSwitch)?.also { terminal.muted("Previous $installerSwitch: $it") },
                convert = { input ->
                    isInstallerSwitchValid(switch = input, aSwitch = installerSwitch, canBeBlank = !isRequired)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input)
                }
            )?.takeIf(String::isNotBlank)?.trim()
            println()
        }
    }

    private fun isInstallerSwitchValid(
        switch: String,
        aSwitch: Switch,
        canBeBlank: Boolean = false
    ): String? {
        val (minBoundary, maxBoundary) = 1 to if (aSwitch == Switch.Custom) 2048 else 512
        return when {
            switch.isBlank() && !canBeBlank -> Errors.blankInput(aSwitch.toString())
            switch.length > maxBoundary -> Errors.invalidLength(min = minBoundary, max = maxBoundary)
            else -> null
        }
    }

    private fun getPreviousValue(aSwitch: Switch): String? {
        return previousInstallerData?.let {
            when (aSwitch) {
                Switch.Silent -> {
                    it.installerSwitches?.silent
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches
                            ?.silent
                }
                Switch.SilentWithProgress -> {
                    it.installerSwitches?.silentWithProgress
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches
                            ?.silentWithProgress
                }
                Switch.Custom -> {
                    it.installerSwitches?.custom
                        ?: it.installers.getOrNull(allManifestData.installers.size)
                            ?.installerSwitches
                            ?.custom
                }
            }
        }
    }

    private fun switchInfo(
        installerType: InstallerManifest.Installer.InstallerType?,
        aSwitch: Switch
    ): Pair<String, TextColors> = with(allManifestData) {
        val isRequired = installerType == InstallerManifest.Installer.InstallerType.EXE &&
            aSwitch != Switch.Custom
        return buildString {
            append(
                if (installerType == InstallerManifest.Installer.InstallerType.EXE && aSwitch != Switch.Custom) {
                    Prompts.required
                } else {
                    Prompts.optional
                }
            )
            append(" Enter the ${aSwitch.toString().lowercase()} install switch")
        } to if (getPreviousValue(aSwitch).isNullOrBlank() && isRequired) brightGreen else brightYellow
    }

    private fun switchExample(aSwitch: Switch): String {
        return buildString {
            append("Example: ")
            append(
                when (aSwitch) {
                    Switch.Silent -> "/S, -verysilent, /qn, --silent, /exenoui."
                    Switch.SilentWithProgress -> "/S, -silent, /qb, /exebasicui."
                    Switch.Custom -> "/norestart, -norestart"
                }
            )
        }
    }
}
