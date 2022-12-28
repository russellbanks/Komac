package input

object YamlExtensions {
    fun String.convertToYamlList(uniqueItems: Boolean = true): List<String>? {
        return if (isNullOrBlank()) {
            null
        } else {
            split(Regex("\\W+")).let { if (uniqueItems) it.distinct() else it }.sorted().filterNot { it.isBlank() }
        }
    }
}
