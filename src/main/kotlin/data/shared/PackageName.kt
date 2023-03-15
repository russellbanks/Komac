package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import detection.files.msi.Msi

class PackageName(private val msi: Msi?, previousPackageName: String?) : TextPrompt {
    override val name: String = "Package name"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val extraText: String = buildString {
        appendLine("Example: Microsoft Teams")
        msi?.productName?.let { appendLine("Detected from MSI: $it") }
    }

    override val default: String? = previousPackageName
}
