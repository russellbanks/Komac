package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules

object PackageIdentifier : TextPrompt {
    override val name: String = "Package Identifier"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 128,
        minLength = 4,
        pattern = Regex("^[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}(\\.[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}){1,7}$"),
        isRequired = true
    )

    override val extraText: String = "Example: Microsoft.Excel"
}
