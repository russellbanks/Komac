package input

enum class LocaleType {
    Installer,
    Package;

    override fun toString() = name.replace("([A-Z])".toRegex(), " $1").trim()
}
