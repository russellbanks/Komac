package data

enum class VersionUpdateState {
    NewPackage,
    NewVersion,
    AddVersion,
    UpdateVersion;

    override fun toString() = name
        .replace("([A-Z])".toRegex(), " $1")
        .trim()
        .lowercase()
        .replaceFirstChar(Char::titlecase)
}
