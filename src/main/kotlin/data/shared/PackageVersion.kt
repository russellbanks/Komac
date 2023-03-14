package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.Prompts
import java.math.BigInteger
import kotlin.random.Random

object PackageVersion : CommandPrompt<String> {
    private const val const = "Package Version"
    private const val versionInfo = "${Prompts.required} Enter the version."
    private const val pattern = "^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"
    const val maxLength = 128
    val regex = Regex(pattern)

    override fun prompt(terminal: Terminal): String? = with(terminal) {
        println(colors.brightGreen(versionInfo))
        info("Example: ${generateRandomVersion()}")
        prompt(const) { input ->
            getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length > maxLength -> Errors.invalidLength(max = maxLength)
            !input.matches(regex) -> Errors.invalidRegex(regex)
            else -> null
        }
    }

    data class VersionPart(val value: BigInteger, val supplement: String, val original: String)

    /**
     * Parses a version part string into a VersionPart object.
     */
    private fun parseVersionPart(part: String): VersionPart {
        val value = part.takeWhile(Char::isDigit).toBigIntegerOrNull() ?: BigInteger.valueOf(0)
        val supplement = part.dropWhile(Char::isDigit)
        return VersionPart(value, supplement, part)
    }

    /**
     * Compares two lists of VersionPart objects lexicographically based on their values and supplements.
     *
     * The comparison is performed element by element in lexicographic order. The first pair of elements that
     * differ determines the result of the comparison. If all elements are equal, the shorter list is considered
     * less than the longer one.
     *
     * @param list1 the first list to compare.
     * @param list2 the second list to compare.
     * @return a negative integer, zero, or a positive integer as list1 is less than, equal to, or greater than
     * list2, respectively.
     */
    private fun compareVersionParts(list1: List<VersionPart>, list2: List<VersionPart>): Int {
        val size = list1.size.coerceAtMost(list2.size)
        return list1.take(size)
            .zip(list2.take(size))
            .map { (leftPart, rightPart) -> compareValuesBy(leftPart, rightPart, { it.value }, { it.supplement }) }
            .firstOrNull { it != 0 }
            ?: list1.size.compareTo(list2.size)
    }

    /**
     * Returns the highest version string from a list of version strings.
     */
    fun List<String>.getHighestVersion(): String? {
        return map { version -> version.split(".").map(::parseVersionPart) }
            .maxWithOrNull(::compareVersionParts)
            ?.joinToString(".") { it.original }
    }

    private fun generateRandomVersion(): String {
        val major = Random.nextInt(1, 10)
        val minor = Random.nextInt(0, 100)
        val patch = Random.nextInt(0, 10)
        return "$major.$minor.$patch"
    }
}
