package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import data.AllManifestData
import data.PreviousManifestData

object PackageName : TextPrompt {
    override val name: String = "Package name"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val extraText: String = buildString {
        appendLine("Example: Microsoft Teams")
        AllManifestData.msi?.productName?.let { appendLine("Detected from MSI: $it") }
    }

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.packageName
}
