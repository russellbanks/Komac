enum class PromptType {
    PackageVersion { override fun toString() = "Version" },
    InstallerUrl { override fun toString() = "Url" },
    Architecture,
    InstallerType { override fun toString() = "Installer Type" },
    SilentSwitch { override fun toString() = "Silent Switch" },
}
