package data.shared

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import java.util.Locale
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest

object Locale {
    private const val PATTERN = "^([a-zA-Z]{2,3}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*$"
    const val DEFAULT_LOCALE = "en-US"

    val example: String get() = "Example: ${Locale.getISOLanguages().random()}-${Locale.getISOCountries().random()}"

    class Installer(
        private val currentInstallerIndex: Int,
        private val previousInstallerManifest: InstallerManifest?
    ) : TextPrompt {
        override val name: String = "Installer locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(PATTERN),
            isRequired = false
        )

        override val default: String? get() = previousInstallerManifest?.run {
            installerLocale ?: installers[currentInstallerIndex].installerLocale
        }

        override val extraText: String = example
    }

    class Package(private val previousDefaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
        override val name: String = "Package locale"

        override val validationRules: ValidationRules = ValidationRules(
            maxLength = 20,
            minLength = 1,
            pattern = Regex(PATTERN),
            isRequired = true
        )

        override val extraText: String = example

        override val default: String get() = previousDefaultLocaleManifest?.packageLocale ?: DEFAULT_LOCALE
    }
}
