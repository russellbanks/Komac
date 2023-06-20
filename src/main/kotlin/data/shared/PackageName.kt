package data.shared

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
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
        append("Example: Microsoft Teams")
        msi?.productName?.let { appendLine("Detected from MSI: $it") }
    }

    override val default: String? get() = defaultLocaleManifest?.packageName
}
