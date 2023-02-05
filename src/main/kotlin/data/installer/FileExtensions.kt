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

object FileExtensions : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.fileExtensionsPrompt() {
        println(colors.brightYellow("${Prompts.optional} $description (Max $maxItems)"))
        installerManifestData.fileExtensions = prompt(
            prompt = const,
            default = getPreviousValue()?.joinToString(", ")?.also {
                muted("Previous file extensions: $it")
            },
            convert = { input ->
                areFileExtensionsValid(input.convertToYamlList())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(uniqueItems)
        println()
    }

    private fun areFileExtensionsValid(fileExtensions: Iterable<String>): String? {
        return when {
            fileExtensions.count() > maxItems -> Errors.invalidLength(max = maxItems)
            fileExtensions.any { !it.matches(regex) } -> {
                Errors.invalidRegex(regex = regex, items = fileExtensions.filterNot { it.matches(regex) })
            }
            fileExtensions.any { it.length > maxItemLength } -> {
                Errors.invalidLength(max = maxItemLength, items = fileExtensions.filter { it.length > maxItemLength })
            }
            else -> null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
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
