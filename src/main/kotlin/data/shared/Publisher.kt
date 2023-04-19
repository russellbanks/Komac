package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import commands.interfaces.ValidationRules
import data.PreviousManifestData

object Publisher : TextPrompt {

    override val name: String = "Publisher"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.publisher

    override val extraText: String = "Example: Microsoft Corporation"

    object Url : UrlPrompt {
        override val name: String = "Publisher url"

        override val previousUrl = PreviousManifestData.defaultLocaleManifest?.publisherUrl

        override val description: String = "publisher home page"
    }
}
