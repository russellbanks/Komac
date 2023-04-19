package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import commands.interfaces.ValidationRules
import data.PreviousManifestData

object License : TextPrompt {
    override val name = "License"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = true
    )

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.license

    override val extraText: String = "Example: MIT, GPL-3.0, Freeware, Proprietary"

    object Url : UrlPrompt {
        override val name: String = "License url"

        override val previousUrl: io.ktor.http.Url? get() = PreviousManifestData.defaultLocaleManifest?.licenseUrl

        override val description: String = "license page url"
    }
}
