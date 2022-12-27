package input

enum class PromptType {
    Architecture,
    Author,
    Commands,
    Copyright,
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
    Scope,
    SilentSwitch,
    SilentWithProgressSwitch,
    Tags;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
