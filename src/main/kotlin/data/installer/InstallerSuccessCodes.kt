package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import extensions.YamlExtensions.convertToList
import input.Prompts
import schemas.manifest.InstallerManifest

class InstallerSuccessCodes(
    private val previousInstallerManifest: InstallerManifest?,
    private val installerSize: Int
) : CommandPrompt<List<Long>> {
    override fun prompt(terminal: Terminal): List<Long>? = with(terminal) {
        println(colors.brightYellow(installerSuccessCodeInfo))
        info(installerSuccessCodesExample)
        return prompt(
            prompt = const,
            default = getPreviousValue()?.also { muted("Previous success codes: $it") }
        ) { input ->
            getError(input)
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(convertToInstallerCodeList(input.trim()))
        }
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
                    items = convertedInput
                        .filter { it < Int.MIN_VALUE || it > UInt.MAX_VALUE.toLong() }
                        .map(Long::toString)
                )
            }
            else -> null
        }
    }

    private fun convertToInstallerCodeList(input: String?): List<Long>? {
        return input?.trim()
            ?.convertToList(uniqueItems)
            ?.mapNotNull(String::toLongOrNull)
            ?.filterNot { it == 0L }
    }

    private fun generateRandomInstallerSuccessCodes(): List<Int> {
        val installerSuccessCodes = listOf(13, 87, 120, 1259, 3010) + IntRange(1601, 1616) + IntRange(1618, 1654)
        return installerSuccessCodes.shuffled().take(3).sorted()
    }

    private fun getPreviousValue(): List<Long>? {
        return previousInstallerManifest?.let {
            it.installerSuccessCodes ?: it.installers.getOrNull(installerSize)?.installerSuccessCodes
        }
    }

    private val installerSuccessCodesExample = "Example: ${generateRandomInstallerSuccessCodes().joinToString(", ")}"

    companion object {
        private const val maxItems = 16
        private const val uniqueItems = true
        private const val const = "Installer Success Codes"
        private const val description =
            "List of additional non-zero installer success exit codes other than known default values by winget"
        private const val installerSuccessCodeInfo = "${Prompts.optional} $description (Max $maxItems)"
    }
}
