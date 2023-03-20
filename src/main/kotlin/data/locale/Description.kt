package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules
import detection.files.msix.Msix

class Description {
    class Short(msix: Msix?) : TextPrompt {
        override val name: String = "Short description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 256,
            isRequired = true
        )

        override val extraText: String? = msix?.description?.let { "Description from installer: $it" }
    }

    class Long(previousDescription: String?) : TextPrompt {
        override val name: String = "Description"

        override val validationRules: ValidationRules = ValidationRules(
            minLength = 3,
            maxLength = 10_000
        )

        override val default: String? = previousDescription
    }
}
