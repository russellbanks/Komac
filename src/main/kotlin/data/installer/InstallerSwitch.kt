package data.installer

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import data.ManifestData
import data.PreviousManifestData

sealed class InstallerSwitch {
    object Silent : TextPrompt {
        override val name: String = "Silent installer switch"
        override val extraText: String = "Example: /S, -verysilent, /qn, --silent, /exenoui"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 512,
            minLength = 1,
            isRequired = true
        )
        override val default: String? = PreviousManifestData.installerManifest?.run {
            installerSwitches?.silent ?: installers.getOrNull(ManifestData.installers.size)?.installerSwitches?.silent
        }
    }

    object SilentWithProgress : TextPrompt {
        override val name: String = "Silent with progress installer switch"
        override val extraText: String = "Example: /S, -silent, /qb, /exebasicui"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 512,
            minLength = 1,
            isRequired = true
        )
        override val default: String? = PreviousManifestData.installerManifest?.run {
            installerSwitches?.silentWithProgress
                ?: installers.getOrNull(ManifestData.installers.size)?.installerSwitches?.silentWithProgress
        }
    }

    object Custom : TextPrompt {
        override val name: String = "Custom installer switch"
        override val extraText: String = "Example: /norestart, -norestart"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 2048,
            minLength = 1,
            isRequired = false
        )
        override val default: String? = PreviousManifestData.installerManifest?.run {
            installerSwitches?.custom ?: installers.getOrNull(ManifestData.installers.size)?.installerSwitches?.custom
        }
    }
}
