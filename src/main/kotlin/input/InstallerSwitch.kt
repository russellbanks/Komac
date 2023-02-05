package input

import org.koin.core.component.KoinComponent

enum class InstallerSwitch : KoinComponent {
    Silent,
    SilentWithProgress,
    Custom;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
