package data

enum class VersionUpdateState {
    NewPackage,
    NewVersion,
    AddVersion,
    UpdateVersion;

    override fun toString() = name
        .replace("(?<=.)(?=\\p{Lu})".toRegex(), " ")
        .lowercase()
        .replaceFirstChar(Char::titlecaseChar)
}
