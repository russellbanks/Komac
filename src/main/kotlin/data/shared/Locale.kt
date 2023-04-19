package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import data.AllManifestData
import data.PreviousManifestData
import java.util.Locale

object Locale {
    private const val pattern = "^([a-zA-Z]{2,3}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*$"
    const val defaultLocale = "en-US"

    val example: String get() = "Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}"

    object Installer : TextPrompt {
        override val name: String = "Installer locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(pattern),
            isRequired = false
        )

        override val default: String? get() = PreviousManifestData.installerManifest?.run {
            installerLocale ?: installers[AllManifestData.installers.size].installerLocale
        }

        override val extraText: String = example
    }

    object Package : TextPrompt {
        override val name: String = "Package locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(pattern),
            isRequired = true
        )

        override val extraText: String = example

        override val default: String get() = PreviousManifestData.defaultLocaleManifest?.packageLocale ?: defaultLocale
    }
}
