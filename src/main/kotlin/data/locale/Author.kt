package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules

class Author(previousAuthor: String?) : TextPrompt {
    override val name: String = "Author"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = false
    )

    override val default: String? = previousAuthor
}
