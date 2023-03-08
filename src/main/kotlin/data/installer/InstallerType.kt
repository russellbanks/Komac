package data.installer

import Errors
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import schemas.manifest.InstallerManifest

class InstallerType(
    private val previousInstaller: InstallerManifest?,
    private val installersSize: Int
) : CommandPrompt<InstallerManifest.Installer.InstallerType> {
    private val installerTypesEnum = InstallerManifest.InstallerType.values().map { it.toString() }

    override fun prompt(terminal: Terminal): InstallerManifest.Installer.InstallerType? = with(terminal) {
        installerTypeInfo().also { (info, infoColor) -> println(infoColor(info)) }
        info("Options: ${installerTypesEnum.joinToString(", ")}")
        return prompt(
            prompt = const,
            default = getPreviousValue()?.toInstallerType()?.also { muted("Previous installer type: $it") }
        ) { input ->
            getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.toInstallerType())
        }
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

    private fun getPreviousValue(): String? {
        return previousInstaller?.let {
            it.installerType?.toString() ?: it.installers.getOrNull(installersSize)?.installerType?.toString()
        }
    }

    private fun installerTypeInfo(): Pair<String, TextColors> {
        return buildString {
            append(if (getPreviousValue() == null) Prompts.required else Prompts.optional)
            append(" Enter the installer type")
        } to if (getPreviousValue() == null) brightGreen else brightYellow
    }

    companion object {
        const val const = "Installer Type"
    }
}
