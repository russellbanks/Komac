package input

enum class LocaleType {
    Installer,
    Package;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
