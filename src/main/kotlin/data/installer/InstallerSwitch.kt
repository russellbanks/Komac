package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.InstallerSwitch
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerSwitch : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.installerSwitchPrompt(installerSwitch: InstallerSwitch) {
        val isRequired = installerManifestData.installerType == InstallerManifest.Installer.InstallerType.EXE &&
            installerSwitch != InstallerSwitch.Custom
        do {
            switchInfo(installerManifestData.installerType, installerSwitch).also { (info, infoColor) ->
                println(infoColor(info))
            }
            println(cyan(switchExample(installerSwitch)))
            val input: String? = prompt(
                prompt = brightWhite(installerSwitch.toString()),
                default = getPreviousValue(installerSwitch)?.also {
                    println(gray("Previous $installerSwitch: $it"))
                }
            )?.trim()
            val (switchValid, error) = isInstallerSwitchValid(
                switch = input,
                installerSwitch = installerSwitch,
                canBeBlank = !isRequired
            )
            error?.let { println(brightRed(it)) }
            if (switchValid == Validation.Success) {
                input?.let { installerManifestData.installerSwitches[installerSwitch] = it.ifBlank { null } }
            }
            println()
        } while (switchValid != Validation.Success)
    }

    fun isInstallerSwitchValid(
        switch: String?,
        installerSwitch: InstallerSwitch,
        canBeBlank: Boolean = false,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val (minBoundary, maxBoundary) = installerSwitch.getLengthBoundary(installerSchema)
        return when {
            switch.isNullOrBlank() && !canBeBlank -> {
                Validation.Blank to Errors.blankInput(installerSwitch.toPromptType())
            }
            (switch?.length ?: 0) > maxBoundary -> {
                Validation.InvalidLength to Errors.invalidLength(min = minBoundary, max = maxBoundary)
            }
            else -> Validation.Success to null
        }
    }

    private fun getPreviousValue(installerSwitch: InstallerSwitch): String? {
        return previousManifestData.remoteInstallerData?.let {
            when (installerSwitch) {
                InstallerSwitch.Silent -> {
                    it.installerSwitches?.silent ?: it.installers[installerManifestData.installers.size]
                        .installerSwitches?.silent
                }
                InstallerSwitch.SilentWithProgress -> {
                    it.installerSwitches?.silentWithProgress ?: it.installers[installerManifestData.installers.size]
                        .installerSwitches?.silentWithProgress
                }
                InstallerSwitch.Custom -> {
                    it.installerSwitches?.custom ?: it.installers[installerManifestData.installers.size]
                        .installerSwitches?.custom
                }
            }
        }
    }

    private fun switchInfo(
        installerType: InstallerManifest.Installer.InstallerType?,
        installerSwitch: InstallerSwitch
    ): Pair<String, TextColors> {
        val isRequired = installerManifestData.installerType == InstallerManifest.Installer.InstallerType.EXE &&
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
