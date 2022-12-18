package input

enum class PromptType {
    PackageIdentifier,
    PackageVersion,
    InstallerUrl,
    Architecture,
    InstallerType,
    SilentSwitch,
    SilentWithProgressSwitch,
    CustomSwitch,
    InstallerLocale,
    ProductCode,
    ReleaseDate,
    FileExtensions,
    Protocols,
    Commands;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
