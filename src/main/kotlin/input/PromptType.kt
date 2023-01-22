package input

enum class PromptType {
    InstallerLocale,
    InstallerUrl,
    PackageLocale;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
