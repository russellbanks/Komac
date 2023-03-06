package utils

fun String.updateVersionInString(previousVersions: List<String>?, newVersion: String): String {
    return previousVersions?.joinToString("|") { it }
        ?.let { replaceFirst(Regex(it), newVersion) }
        ?: this
}
