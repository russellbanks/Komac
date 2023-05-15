package data.installer

import Errors
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.ManifestData
import data.PreviousManifestData
import input.Prompts
import input.Switch
import schemas.manifest.InstallerManifest

object InstallerSwitch {
    fun installerSwitchPrompt(installerSwitch: Switch, terminal: Terminal) = with(ManifestData) {
        if (
            installerType == InstallerManifest.InstallerType.EXE ||
            installerSwitch == Switch.Custom
        ) {
            val isRequired = installerType == InstallerManifest.InstallerType.EXE &&
                installerSwitch != Switch.Custom
            switchInfo(installerType, installerSwitch).also { (info, infoColor) ->
                println(infoColor(info))
            }
            terminal.info(switchExample(installerSwitch))
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

    private fun getPreviousValue(switch: Switch): String? = PreviousManifestData.installerManifest?.run {
        when (switch) {
            Switch.Silent -> installerSwitches?.silent
                ?: installers.getOrNull(ManifestData.installers.size)
                    ?.installerSwitches
                    ?.silent
            Switch.SilentWithProgress -> installerSwitches?.silentWithProgress
                ?: installers.getOrNull(ManifestData.installers.size)
                    ?.installerSwitches
                    ?.silentWithProgress
            Switch.Custom -> installerSwitches?.custom
                ?: installers.getOrNull(ManifestData.installers.size)
                    ?.installerSwitches
                    ?.custom
        }
    }

    private fun switchInfo(
        installerType: InstallerManifest.InstallerType?,
        aSwitch: Switch
    ): Pair<String, TextColors> {
        val isRequired = installerType == InstallerManifest.InstallerType.EXE && aSwitch != Switch.Custom
        return buildString {
            append(
                if (installerType == InstallerManifest.InstallerType.EXE && aSwitch != Switch.Custom) {
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
