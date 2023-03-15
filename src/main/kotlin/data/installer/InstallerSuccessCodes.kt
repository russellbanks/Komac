package data.installer

import Errors
import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import extensions.YamlExtensions.convertToList
import schemas.manifest.InstallerManifest

class InstallerSuccessCodes(
    previousInstallerManifest: InstallerManifest?,
    private val installerSize: Int
) : ListPrompt<Long> {
    override val name: String = "Installer success codes"

    override val description: String =
        "List of additional non-zero installer success exit codes other than known default values by winget"

    override val extraText: String = "Example: ${generateRandomInstallerSuccessCodes().joinToString()}"

    override val default: List<Long>? = previousInstallerManifest?.run {
        installerSuccessCodes ?: installers.getOrNull(installerSize)?.installerSuccessCodes
    }

    override val validationRules: ListValidationRules<Long> = ListValidationRules(
        maxItems = 16,
        minItemLength = 1,
        transform = ::convertToInstallerCodeList,
        additionalValidation = { longList ->
            if (longList.any { it < Int.MIN_VALUE.toLong() || it > UInt.MAX_VALUE.toLong() }) {
                Errors.invalidLength(
                    min = Int.MIN_VALUE,
                    max = UInt.MAX_VALUE.toLong(),
                    items = longList.filter { it < Int.MIN_VALUE || it > UInt.MAX_VALUE.toLong() }.map(Long::toString)
                )
            } else {
                null
            }
        }
    )

    private fun convertToInstallerCodeList(input: String): List<Long> {
        return convertToList(input).mapNotNull(String::toLongOrNull).filterNot { it == 0L }
    }

    private fun generateRandomInstallerSuccessCodes(): List<Int> {
        val installerSuccessCodes = listOf(13, 87, 120, 1259, 3010) + IntRange(1601, 1616) + IntRange(1618, 1654)
        return installerSuccessCodes.shuffled().take(3).sorted()
    }
}
