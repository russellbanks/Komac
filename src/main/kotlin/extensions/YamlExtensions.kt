package extensions

object YamlExtensions {
    fun convertToList(string: String): List<String> = string.trim()
        .split("\\W+".toRegex())
        .filterNot(String::isBlank)
        .toSortedSet()
        .toList()
}
