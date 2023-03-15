package extensions

object YamlExtensions {
    fun convertToList(string: String): List<String> {
        return string.trim()
            .split("\\W+".toRegex())
            .filterNot(String::isBlank)
            .toSet()
            .sorted()
    }
}
