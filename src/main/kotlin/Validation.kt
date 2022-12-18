enum class Validation {
    Blank,
    InvalidArchitecture,
    InvalidInstallerScope,
    InvalidInstallerType,
    InvalidLength,
    InvalidPattern,
    InvalidReleaseDate,
    InvalidUpgradeBehaviour,
    Success,
    UnsuccessfulResponseCode;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
