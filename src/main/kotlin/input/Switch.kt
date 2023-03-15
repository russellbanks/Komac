package input

enum class Switch {
    Silent,
    SilentWithProgress,
    Custom;

    override fun toString() = name
        .replace("(?<=.)(?=\\p{Lu})".toRegex(), " ")
        .lowercase()
        .replaceFirstChar(Char::titlecaseChar)
}
