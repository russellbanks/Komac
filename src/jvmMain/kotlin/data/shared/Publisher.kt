package data.shared

import commands.prompts.TextPrompt
import io.menu.prompts.UrlPrompt
import commands.prompts.validation.ValidationRules
import schemas.manifest.DefaultLocaleManifest

class Publisher(private val defaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name: String = "Publisher"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val default: String? get() = defaultLocaleManifest?.publisher

    override val extraText: String = "Example: Microsoft Corporation"

    class Url(defaultLocaleManifest: DefaultLocaleManifest?) : UrlPrompt {
        override val name: String = "Publisher url"

        override val previousUrl = defaultLocaleManifest?.publisherUrl

        override val description: String = "publisher home page"
    }
}
