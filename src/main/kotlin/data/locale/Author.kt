package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import data.PreviousManifestData

object Author : TextPrompt {
    override val name: String = "Author"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = false
    )

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.author
}
