package input

enum class PromptType {
    Architecture,
    Author,
    Commands,
    CustomSwitch,
    FileExtensions,
    InstallModes,
    InstallerLocale,
    InstallerSuccessCodes,
    InstallerType,
    InstallerUrl,
    License,
    Moniker,
    PackageIdentifier,
    PackageLocale,
    PackageName,
    PackageVersion,
    ProductCode,
    Protocols,
    Publisher,
    ReleaseDate,
    SilentSwitch,
    SilentWithProgressSwitch;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
