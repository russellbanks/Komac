package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.InstallerSwitch
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerSwitch : KoinComponent {
    fun Terminal.installerSwitchPrompt(installerSwitch: InstallerSwitch) {
        val installerManifestData: InstallerManifestData by inject()
        val isRequired = installerManifestData.installerType == InstallerManifest.InstallerType.EXE &&
            installerSwitch != InstallerSwitch.Custom
        do {
            val infoTextColour = if (isRequired) brightGreen else brightYellow
            println(infoTextColour(switchInfo(installerManifestData.installerType, installerSwitch)))
            var switchResponse: String? = null
            when (installerSwitch) {
                InstallerSwitch.Silent -> installerManifestData.silentSwitch = prompt(
                    brightWhite(PromptType.SilentSwitch.toString())
                )?.trim().also { switchResponse = it }
                InstallerSwitch.SilentWithProgress -> {
                    installerManifestData.silentWithProgressSwitch = prompt(
                        brightWhite(PromptType.SilentWithProgressSwitch.toString())
                    )?.trim().also { switchResponse = it }
                }
                InstallerSwitch.Custom -> installerManifestData.customSwitch = prompt(
                    brightWhite(PromptType.CustomSwitch.toString())
                )?.trim().also { switchResponse = it }
            }
            val (switchValid, error) = isInstallerSwitchValid(
                switch = switchResponse,
                installerSwitch = installerSwitch,
                canBeBlank = !isRequired
            )
            error?.let { println(red(it)) }
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

    private fun switchInfo(installerType: InstallerManifest.InstallerType?, installerSwitch: InstallerSwitch): String {
        return buildString {
            append(
                when {
                    installerType == InstallerManifest.InstallerType.EXE &&
                        installerSwitch != InstallerSwitch.Custom -> Prompts.required
                    else -> Prompts.optional
                }
            )
            append(" Enter the ${installerSwitch.toString().lowercase()}. For example: ")
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
