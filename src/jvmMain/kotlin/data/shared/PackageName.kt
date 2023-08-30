package data.shared

import commands.prompts.TextPrompt
import commands.prompts.validation.ValidationRules
import schemas.manifest.DefaultLocaleManifest
import utils.msi.Msi

class PackageName(private val msi: Msi?, private val defaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name: String = "Package name"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val extraText: String = buildString {
        msi?.productName?.let {
            appendLine(EXAMPLE)
            append("Detected from MSI: $it")
        } ?: append(EXAMPLE)
    }

    override val default: String? get() = defaultLocaleManifest?.packageName

    companion object {
        private const val EXAMPLE = "Example: Microsoft Teams"
    }
}
