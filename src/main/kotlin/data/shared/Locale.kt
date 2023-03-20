package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import schemas.manifest.InstallerManifest
import java.util.Locale

object Locale {
    private const val pattern = "^([a-zA-Z]{2,3}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*$"
    const val defaultLocale = "en-US"

    val example: String
        get() = "Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}"

    class Installer(
        previousInstallerManifest: InstallerManifest?,
        private val installersSize: Int
    ) : TextPrompt {
        override val name: String = "Installer locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(pattern),
            isRequired = false
        )

        override val default: String? = previousInstallerManifest?.run {
            installerLocale ?: installers[installersSize].installerLocale
        }

        override val extraText: String = example
    }

    class Package(previousPackageLocale: String?) : TextPrompt {
        override val name: String = "Package locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(pattern),
            isRequired = true
        )

        override val extraText: String = example

        override val default: String = previousPackageLocale ?: defaultLocale
    }
}
