package input

enum class Switch {
    Silent,
    SilentWithProgress,
    Custom;

    override fun toString() = name.replace("([A-Z])".toRegex(), " $1").trim()
}
