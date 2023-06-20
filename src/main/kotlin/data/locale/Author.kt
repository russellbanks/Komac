package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import schemas.manifest.DefaultLocaleManifest

class Author(private val previousDefaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name: String = "Author"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = false
    )

    override val default: String? get() = previousDefaultLocaleManifest?.author
}
