package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules

class Moniker(previousMoniker: String?) : TextPrompt {
    override val name = "Moniker"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 40,
        minLength = 1,
        isRequired = false
    )

    override val default: String? = previousMoniker

    override val extraText: String = "Example: vscode"
}
