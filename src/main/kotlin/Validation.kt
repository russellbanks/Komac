enum class Validation {
    InvalidArchitecture,
    InvalidInstallerScope,
    InvalidInstallerType,
    InvalidInstallMode,
    InvalidLength,
    InvalidPattern,
    InvalidUpgradeBehaviour,
    Success,
    UnsuccessfulResponseCode;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
