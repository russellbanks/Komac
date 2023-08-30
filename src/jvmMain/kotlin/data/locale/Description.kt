package data.locale

import commands.prompts.TextPrompt
import commands.prompts.validation.ValidationRules
import schemas.manifest.DefaultLocaleManifest
import utils.msix.Msix

object Description {
    class Short(private val msix: Msix?) : TextPrompt {
        override val name: String = "Short description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 256,
            isRequired = true
        )

        override val extraText: String? get() = msix?.description?.let {
            "Description from installer: $it"
        }
    }

    class Long(private val defaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
        override val name: String = "Description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 10_000,
            isRequired = false
        )

        override val default: String? get() = defaultLocaleManifest?.description
    }
}
