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

object Protocols : KoinComponent, CommandPrompt<List<String>> {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<String> = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = InstallerManifest::protocols.name.replaceFirstChar { it.titlecase() },
            default = getPreviousValue()?.also { muted("Previous protocols: $it") },
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
            convertedInput.any { it.length > maxLength } -> {
                Errors.invalidLength(max = maxLength, items = convertedInput.filter { it.length > maxLength })
            }
            else -> null
        }
    }

    private suspend fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.protocols ?: it.installers.getOrNull(installerManifestData.installers.size)?.protocols
        }
    }

    private const val maxItems = 64
    private const val maxLength = 2048
    private const val uniqueItems = true
    private const val description = "List of protocols the package provides a handler for"
}
