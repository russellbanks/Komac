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

object InstallerSuccessCodes : KoinComponent, CommandPrompt<List<Long>> {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<Long> = with(terminal) {
        println(colors.brightYellow(installerSuccessCodeInfo))
        info(installerSuccessCodesExample)
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous success codes: $it") },
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(convertToInstallerCodeList(input.trim()))
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        val convertedInput = convertToInstallerCodeList(input?.trim())
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxItems -> Errors.invalidLength(max = maxItems)
            convertedInput.any { it < Int.MIN_VALUE.toLong() || it > UInt.MAX_VALUE.toLong() } -> {
                Errors.invalidLength(
                    min = Int.MIN_VALUE,
                    max = UInt.MAX_VALUE.toLong(),
                    items = convertedInput.filter {
                        it < Int.MIN_VALUE || it > UInt.MAX_VALUE.toLong()
                    }.map { it.toString() }
                )
            }
            else -> null
        }
    }

    private fun convertToInstallerCodeList(input: String?): List<Long>? {
        return input?.trim()
            ?.convertToYamlList(uniqueItems)
            ?.mapNotNull { it.toLongOrNull() }
            ?.filterNot { it == 0L }
    }

    private fun generateRandomInstallerSuccessCodes(): List<Int> {
        val installerSuccessCodes = listOf(13, 87, 120, 1259, 3010) + IntRange(1601, 1616) + IntRange(1618, 1654)
        return installerSuccessCodes.shuffled().take(3).sorted()
    }

    private suspend fun getPreviousValue(): List<Long>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.installerSuccessCodes
                ?: it.installers.getOrNull(installerManifestData.installers.size)?.installerSuccessCodes
        }
    }

    private const val maxItems = 16
    private const val uniqueItems = true
    private const val const = "Installer Success Codes"
    private const val description =
        "List of additional non-zero installer success exit codes other than known default values by winget"
    private const val installerSuccessCodeInfo = "${Prompts.optional} $description (Max $maxItems)"
    private val installerSuccessCodesExample =
        "Example: ${generateRandomInstallerSuccessCodes().joinToString(", ")}"
}
