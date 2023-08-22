package data

enum class VersionUpdateState {
    NewPackage,
    NewVersion,
    AddVersion,
    UpdateVersion,
    RemoveVersion;

    override fun toString() = name
        .replace("(?<=.)(?=\\p{Lu})".toRegex(), " ")
        .lowercase()
        .replaceFirstChar(Char::titlecase)
}
