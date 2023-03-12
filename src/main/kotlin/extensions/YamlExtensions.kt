package extensions

object YamlExtensions {
    fun String.convertToList(uniqueItems: Boolean = true): List<String> {
        return split("\\W+".toRegex())
            .filterNot(String::isBlank)
            .let { if (uniqueItems) it.distinct() else it }
            .sorted()
    }
}
