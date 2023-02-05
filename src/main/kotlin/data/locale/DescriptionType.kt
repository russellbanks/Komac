package data.locale

enum class DescriptionType(val promptName: String, val minLength: Int, val maxLength: Int) {
    Short("Short Description", 3, 256),
    Long("Description", 3, 10000)
}
