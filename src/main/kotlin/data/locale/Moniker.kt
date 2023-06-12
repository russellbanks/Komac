package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import data.PreviousManifestData

object Moniker : TextPrompt {
    override val name = "Moniker"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 40,
        minLength = 1,
        isRequired = false
    )

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.moniker

    override val extraText: String = "Example: vscode"
}
