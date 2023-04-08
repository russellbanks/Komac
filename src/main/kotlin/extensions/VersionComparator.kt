package extensions

import java.math.BigInteger

private fun parseVersionPart(part: String): VersionPart {
    val value = part.takeWhile(Char::isDigit).toBigIntegerOrNull() ?: BigInteger.ZERO
    val supplement = part.dropWhile(Char::isDigit)
    return VersionPart(value, supplement)
}

val versionStringComparator = Comparator<String> { version1, version2 ->
    val versionParts1 = version1.split('.').map(::parseVersionPart)
    val versionParts2 = version2.split('.').map(::parseVersionPart)
    compareVersionParts(versionParts1, versionParts2)
}

private fun compareVersionParts(list1: List<VersionPart>, list2: List<VersionPart>): Int {
    val minLength = minOf(list1.size, list2.size)

    for (index in 0 until minLength) {
        val comparisonResult = compareValuesBy(
            list1[index], list2[index],
            VersionPart::value, VersionPart::supplement
        )
        if (comparisonResult != 0) {
            return comparisonResult
        }
    }

    return list1.size.compareTo(list2.size)
}

data class VersionPart(val value: BigInteger, val supplement: String)