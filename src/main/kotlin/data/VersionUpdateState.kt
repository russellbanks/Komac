package data

enum class VersionUpdateState {
    NewPackage,
    NewVersion,
    AddVersion,
    UpdateVersion;

    override fun toString() = name
        .replace(Regex("([A-Z])"), " $1")
        .trim()
        .lowercase()
        .replaceFirstChar { it.titlecase() }
}
