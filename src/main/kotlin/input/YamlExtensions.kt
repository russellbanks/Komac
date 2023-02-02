package input

object YamlExtensions {
    fun String.convertToYamlList(uniqueItems: Boolean = true): List<String> {
        return split(Regex("\\W+")).let { if (uniqueItems) it.distinct() else it }.sorted().filterNot { it.isBlank() }
    }
}
