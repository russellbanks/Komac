package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.UrlPrompt
import io.menu.prompts.ValidationRules
import schemas.manifest.DefaultLocaleManifest

class License(private val defaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name = "License"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = true
    )

    override val default: String? get() = defaultLocaleManifest?.license

    override val extraText: String = "Example: MIT, GPL-3.0, Freeware, Proprietary"

    class Url(private val defaultLocaleManifest: DefaultLocaleManifest?) : UrlPrompt {
        override val name: String = "License url"

        override val previousUrl: io.ktor.http.Url? get() = defaultLocaleManifest?.licenseUrl

        override val description: String = "license page url"
    }
}
