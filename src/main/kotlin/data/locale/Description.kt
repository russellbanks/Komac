package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import data.ManifestData
import data.PreviousManifestData

object Description {
    object Short : TextPrompt {
        override val name: String = "Short description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 256,
            isRequired = true
        )

        override val extraText: String? get() = ManifestData.msix?.description?.let {
            "Description from installer: $it"
        }
    }

    object Long : TextPrompt {
        override val name: String = "Description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 10_000,
            isRequired = false
        )

        override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.description
    }
}
