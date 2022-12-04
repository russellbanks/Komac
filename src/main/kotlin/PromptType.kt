enum class PromptType {
    PackageVersion { override fun toString() = "Version" },
    InstallerUrl { override fun toString() = "Url" }
}