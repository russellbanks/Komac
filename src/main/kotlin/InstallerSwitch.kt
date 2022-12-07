enum class InstallerSwitch {
    Silent,
    SilentWithProgress { override fun toString() = "Silent with Progress" },
    Custom
}
