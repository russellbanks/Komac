package utils

import space.kscience.kmath.operations.BigInt
import space.kscience.kmath.operations.parseBigInteger

private fun parseVersionPart(part: String): VersionPart {
    val (valueStr, supplement) = part.partition(Char::isDigit)
    val value = valueStr.parseBigInteger() ?: BigInt.ZERO
    return VersionPart(value, supplement)
}

val versionStringComparator: Comparator<String> = Comparator { version1, version2 ->
    val versionParts1 = version1.split('.').map(::parseVersionPart)
    val versionParts2 = version2.split('.').map(::parseVersionPart)
    compareVersionParts(versionParts1, versionParts2)
}

private fun compareVersionParts(parts1: List<VersionPart>, parts2: List<VersionPart>): Int {
    val minLength = minOf(parts1.size, parts2.size)
    val comparisonResult = (0..<minLength).firstNotNullOfOrNull { index ->
        compareValuesBy(parts1[index], parts2[index], VersionPart::value, VersionPart::supplement).takeIf { it != 0 }
    }
    return comparisonResult ?: parts1.size.compareTo(parts2.size)
}

data class VersionPart(val value: BigInt, val supplement: String)
