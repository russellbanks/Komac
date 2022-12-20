package input

enum class PromptType {
    Architecture,
    PackageName,
    Publisher,
    Commands,
    CustomSwitch,
    FileExtensions,
    InstallerLocale,
    InstallerSuccessCodes,
    InstallerType,
    InstallerUrl,
    InstallModes,
    Moniker,
    PackageIdentifier,
    PackageLocale,
    PackageVersion,
    ProductCode,
    Protocols,
    ReleaseDate,
    SilentSwitch,
    SilentWithProgressSwitch;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
