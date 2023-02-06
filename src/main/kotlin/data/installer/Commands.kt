package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.InstallerManifestData
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object Commands : KoinComponent, CommandPrompt<List<String>> {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<String> = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = InstallerManifest::commands.name.replaceFirstChar { it.titlecase() },
            default = getPreviousValue()?.also { muted("Previous commands: $it") },
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().convertToYamlList(uniqueItems))
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToYamlList(uniqueItems)
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxItems -> Errors.invalidLength(max = maxItems)
            convertedInput.any { it.length > maxItemLength } -> {
                Errors.invalidLength(
                    min = minItemLength,
                    max = maxItemLength,
                    items = convertedInput.filter { it.length > maxItemLength }
                )
            }
            else -> null
        }
    }

    private suspend fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.commands ?: it.installers.getOrNull(installerManifestData.installers.size)?.commands
        }
    }

    private const val description = "List of commands or aliases to run the package"
    private const val maxItems = 16
    private const val minItemLength = 1
    private const val maxItemLength = 40
    private const val uniqueItems = true
}
