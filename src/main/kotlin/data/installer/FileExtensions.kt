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
import kotlin.system.exitProcess

object FileExtensions : KoinComponent, CommandPrompt<List<String>> {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<String> = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous file extensions: $it") },
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
            convertedInput.any { !it.matches(regex) } -> {
                Errors.invalidRegex(regex = regex, items = convertedInput.filterNot { it.matches(regex) })
            }
            convertedInput.any { it.length > maxItemLength } -> {
                Errors.invalidLength(max = maxItemLength, items = convertedInput.filter { it.length > maxItemLength })
            }
            else -> null
        }
    }

    private suspend fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.fileExtensions ?: it.installers.getOrNull(installerManifestData.installers.size)?.fileExtensions
        }
    }

    private const val const = "File Extensions"
    private const val description = "List of file extensions the package could support"
    private const val maxItems = 512
    private const val maxItemLength = 64
    private const val pattern = "^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"
    private val regex = Regex(pattern)
    private const val uniqueItems = true
}
