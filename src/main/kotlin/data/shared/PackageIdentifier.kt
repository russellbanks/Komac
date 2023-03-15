package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules

object PackageIdentifier : TextPrompt {
    override val name: String = "Package Identifier"

    const val maxLength = 128

    const val minLength = 4

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = maxLength,
        minLength = minLength,
        pattern = Regex("^[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}(\\.[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}){1,7}$"),
        isRequired = true
    )

    override val extraText: String = "Example: Microsoft.Excel"
}
