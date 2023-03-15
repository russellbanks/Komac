package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import java.math.BigInteger

object PackageVersion : TextPrompt {
    private const val pattern = "^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"
    const val maxLength = 128
    val regex = Regex(pattern)

    override val name: String = "Package Version"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 128,
        minLength = 1,
        pattern = Regex(pattern),
        isRequired = true
    )

    override val extraText: String = "Example: ${generateRandomVersion()}"

    /**
     * Parses a version part string into a VersionPart object.
     */
    private fun parseVersionPart(part: String): VersionPart {
        val value = part.takeWhile(Char::isDigit).toBigIntegerOrNull() ?: BigInteger.ZERO
        val supplement = part.dropWhile(Char::isDigit)
        return value to supplement
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
            .zip(list2.take(size)) { left, right -> compareValuesBy(left, right, VersionPart::first, VersionPart::second) }
            .firstOrNull { it != 0 }
            ?: list1.size.compareTo(list2.size)
    }

    /**
     * Returns the highest version string from a list of version strings.
     */
    fun List<String>.getHighestVersion(): String? {
        return map { version -> version.split(".").map(::parseVersionPart) }
            .maxWithOrNull(::compareVersionParts)
            ?.joinToString(".") { (value, supplement) -> "$value$supplement" }
    }

    /**
     * Generates a random version string in the format "major.minor.patch".
     *
     * @return a randomly generated version string.
     */
    private fun generateRandomVersion(): String {
        val major = (1..10).random()
        val minor = (0..99).random()
        val patch = (0..9).random()
        return "$major.$minor.$patch"
    }
}

typealias VersionPart = Pair<BigInteger, String>
