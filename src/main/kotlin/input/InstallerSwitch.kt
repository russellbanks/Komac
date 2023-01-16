package input

import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.SchemasImpl
import schemas.data.InstallerSchema

enum class InstallerSwitch : KoinComponent {
    Silent,
    SilentWithProgress,
    Custom;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()

    fun toPromptType(): PromptType {
        return when (this) {
            Silent -> PromptType.SilentSwitch
            SilentWithProgress -> PromptType.SilentWithProgressSwitch
            Custom -> PromptType.CustomSwitch
        }
    }

    fun getLengthBoundary(
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Int, Int> {
        val installerSwitchProperties = installerSchema.definitions.installerSwitches.properties
        return when (this) {
            Silent -> Pair(installerSwitchProperties.silent.minLength, installerSwitchProperties.silent.maxLength)
            SilentWithProgress -> Pair(
                installerSwitchProperties.silentWithProgress.minLength,
                installerSwitchProperties.silentWithProgress.maxLength
            )
            Custom -> Pair(installerSwitchProperties.custom.minLength, installerSwitchProperties.custom.maxLength)
        }
    }
}
