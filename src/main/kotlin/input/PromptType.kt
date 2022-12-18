package input

enum class PromptType {
    PackageIdentifier { override fun toString() = "Package Identifier" },
    PackageVersion { override fun toString() = "Version" },
    InstallerUrl { override fun toString() = "Url" },
    Architecture,
    InstallerType { override fun toString() = "Installer Type" },
    SilentSwitch { override fun toString() = "Silent Switch" },
    SilentWithProgressSwitch { override fun toString() = "Silent with Progress Switch" },
    CustomSwitch { override fun toString() = "Custom Switch" },
    InstallerLocale { override fun toString() = "Installer Locale" },
    ProductCode { override fun toString() = "Product Code" },
    ReleaseDate { override fun toString() = "Release Date" },
    FileExtensions { override fun toString() = "File Extensions" },
    Protocols
}
