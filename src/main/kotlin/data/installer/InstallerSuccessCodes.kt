package data.installer

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

object InstallerSuccessCodes : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.installerSuccessCodesPrompt() {
        println(colors.brightYellow(installerSuccessCodeInfo))
        info(installerSuccessCodesExample)
        installerManifestData.installerSuccessCodes = prompt(
            prompt = const,
            default = getPreviousValue()?.joinToString(", ")?.also {
                muted("Previous success codes: $it")
            },
            convert = { input ->
                areInstallerSuccessCodesValid(convertToInstallerCodeList(input))
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        ).let { convertToInstallerCodeList(it) }
        println()
    }

    private fun convertToInstallerCodeList(input: String?): List<Long>? {
        return input?.trim()
            ?.convertToYamlList(uniqueItems)
            ?.mapNotNull { it.toLongOrNull() }
            ?.filterNot { it == 0L }
    }

    private fun areInstallerSuccessCodesValid(installerSuccessCodes: List<Long>?): String? {
        return when {
            (installerSuccessCodes?.count() ?: 0) > maxItems -> Errors.invalidLength(max = maxItems)
            installerSuccessCodes?.any {
                it < Int.MIN_VALUE.toLong() || it > UInt.MAX_VALUE.toLong()
            } == true -> {
                Errors.invalidLength(
                    min = Int.MIN_VALUE,
                    max = UInt.MAX_VALUE.toLong(),
                    items = installerSuccessCodes.filter {
                        it < Int.MIN_VALUE || it > UInt.MAX_VALUE.toLong()
                    }.map { it.toString() }
                )
            }
            else -> null
        }
    }

    private fun generateRandomInstallerSuccessCodes(): List<Int> {
        val installerSuccessCodes = listOf(13, 87, 120, 1259, 3010) + IntRange(1601, 1616) + IntRange(1618, 1654)
        return installerSuccessCodes.shuffled().take(3).sorted()
    }

    private fun getPreviousValue(): List<Long>? {
        return previousManifestData.remoteInstallerData?.let {
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
