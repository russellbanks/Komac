enum class Validation {
    InvalidLength,
    InvalidPattern,
    Success,
    UnsuccessfulResponseCode;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
