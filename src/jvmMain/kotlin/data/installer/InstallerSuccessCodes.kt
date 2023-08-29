package data.installer

import Errors
import commands.prompts.ListPrompt
import commands.prompts.validation.ListValidationRules
import schemas.manifest.InstallerManifest
import extensions.YamlExtensions.convertToList

class InstallerSuccessCodes(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
) : ListPrompt<Long> {
    override val name: String = "Installer success codes"

    override val description: String =
        "List of additional non-zero installer success exit codes other than known default values by winget"

    override val extraText: String = "Example: ${randomInstallerSuccessCodes.joinToString()}"

    override val default: List<Long>? get() = previousInstallerManifest?.run {
        installerSuccessCodes ?: installers.getOrNull(currentInstallerIndex)?.installerSuccessCodes
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

    private val randomInstallerSuccessCodes: Sequence<Int>
        get() = (sequenceOf(13, 87, 120, 1259, 3010) + (1601..1616) + (1618..1654))
            .shuffled()
            .take(3)
            .sorted()
}
