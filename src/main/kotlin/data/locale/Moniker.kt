package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import schemas.manifest.DefaultLocaleManifest

class Moniker(private val defaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name = "Moniker"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 40,
        minLength = 1,
        isRequired = false
    )

    override val default: String? get() = defaultLocaleManifest?.moniker

    override val extraText: String = "Example: vscode"
}
