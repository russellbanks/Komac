package data.installer

import Errors
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object InstallerType : KoinComponent, CommandPrompt<InstallerManifest.Installer.InstallerType> {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val installerTypesEnum = InstallerManifest.InstallerType.values().map { it.toString() }

    override suspend fun prompt(terminal: Terminal): InstallerManifest.Installer.InstallerType = with(terminal) {
        installerTypeInfo().also { (info, infoColor) -> println(infoColor(info)) }
        info("Options: ${installerTypesEnum.joinToString(", ")}")
        return prompt(
            prompt = const,
            default = getPreviousValue()?.toInstallerType()?.also { muted("Previous installer type: $it") },
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.toInstallerType())
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            !installerTypesEnum.contains(input) -> Errors.invalidEnum(installerTypesEnum)
            else -> null
        }
    }

    private fun String.toInstallerType(): InstallerManifest.Installer.InstallerType {
        InstallerManifest.Installer.InstallerType.values().forEach { if (it.toString().lowercase() == this) return it }
        throw IllegalArgumentException("Invalid installer type: $this")
    }

    private suspend fun getPreviousValue(): String? = with(allManifestData) {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.installerType?.toString()
                ?: it.installers.getOrNull(installers.size)?.installerType?.toString()
        }
    }

    private suspend fun installerTypeInfo(): Pair<String, TextColors> {
        return buildString {
            append(if (getPreviousValue() == null) Prompts.required else Prompts.optional)
            append(" Enter the installer type")
        } to if (getPreviousValue() == null) brightGreen else brightYellow
    }

    const val const = "Installer Type"
}
