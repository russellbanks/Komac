package data

object FileExtensions {
    fun convertInputToList(input: String?): List<String>? {
        return if (input.isNullOrBlank()) {
            null
        } else if (input.contains(",")) {
            input.split(",").map { it.trim() }
        } else {
            input.split(" ").map { it.trim() }
        }.distinct().sorted()
    }
}
