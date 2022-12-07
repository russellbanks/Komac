enum class PromptType {
    PackageIdentifier { override fun toString(): String = "Package Identifier" },
    PackageVersion { override fun toString() = "Version" },
    InstallerUrl { override fun toString() = "Url" },
    Architecture,
    InstallerType { override fun toString() = "Installer Type" },
    SilentSwitch { override fun toString() = "Silent Switch" },
    SilentWithProgressSwitch { override fun toString() = "Silent with Progress Switch" },
    CustomSwitch { override fun toString() = "Custom Switch" },
    InstallerLocale { override fun toString() = "Installer Locale" }
}
