package data.installer

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import schemas.manifest.InstallerManifest

sealed class InstallerSwitch {
    class Silent(currentInstallerIndex: Int, previousInstallerManifest: InstallerManifest?) : TextPrompt {
        override val name: String = "Silent installer switch"
        override val extraText: String = "Example: /S, -verysilent, /qn, --silent, /exenoui"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 512,
            minLength = 1,
            isRequired = true
        )
        override val default: String? = previousInstallerManifest?.run {
            installerSwitches?.silent ?: installers.getOrNull(currentInstallerIndex)?.installerSwitches?.silent
        }
    }

    class SilentWithProgress(currentInstallerIndex: Int, previousInstallerManifest: InstallerManifest?) : TextPrompt {
        override val name: String = "Silent with progress installer switch"
        override val extraText: String = "Example: /S, -silent, /qb, /exebasicui"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 512,
            minLength = 1,
            isRequired = true
        )
        override val default: String? = previousInstallerManifest?.run {
            installerSwitches?.silentWithProgress
                ?: installers.getOrNull(currentInstallerIndex)?.installerSwitches?.silentWithProgress
        }
    }

    class Custom(currentInstallerIndex: Int, previousInstallerManifest: InstallerManifest?) : TextPrompt {
        override val name: String = "Custom installer switch"
        override val extraText: String = "Example: /norestart, -norestart"
        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 2048,
            minLength = 1,
            isRequired = false
        )
        override val default: String? = previousInstallerManifest?.run {
            installerSwitches?.custom ?: installers.getOrNull(currentInstallerIndex)?.installerSwitches?.custom
        }
    }
}
