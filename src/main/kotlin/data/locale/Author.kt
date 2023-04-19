package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
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
