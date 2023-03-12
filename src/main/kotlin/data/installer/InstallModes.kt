package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.InstallerManifest

class InstallModes(
    private val previousInstallerManifest: InstallerManifest?,
    private val installerSize: Int
) : CommandPrompt<List<InstallerManifest.InstallModes>> {
    override fun prompt(terminal: Terminal): List<InstallerManifest.InstallModes>? = with(terminal) {
        println(colors.brightYellow(installModesInfo))
        info(installModesExample)
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous install modes: $it") }
        ) { input ->
            getError(input)
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(input.trim().convertToList(uniqueItems).toInstallModes())
        }
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToList(uniqueItems)?.toInstallModes()
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxItems -> Errors.invalidLength(max = maxItems)
            convertedInput.any { it !in InstallerManifest.InstallModes.values() } -> {
                Errors.invalidEnum(InstallerManifest.InstallModes.values().map { it.toString() })
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<InstallerManifest.InstallModes>? {
        return previousInstallerManifest?.let { installerManifest ->
            installerManifest.installModes
                ?: installerManifest.installers
                    .getOrNull(installerSize)
                    ?.installModes
                    ?.map { it.toManifestInstallMode() }
        }
    }

    private fun List<String>.toInstallModes(): List<InstallerManifest.InstallModes> {
        return mapNotNull { string ->
            InstallerManifest.InstallModes.values().find { it.name.lowercase() == string.lowercase() }
        }
    }

    companion object {
        private const val installModesInfo = "${Prompts.optional} Enter the list of supported installer modes"
        private const val const = "Install Modes"
        private const val maxItems = 3
        private const val uniqueItems = true
        private val installModesExample = "Options: ${InstallerManifest.InstallModes.values().joinToString(", ")}"
    }
}
