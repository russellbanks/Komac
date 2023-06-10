package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.UrlPrompt
import io.menu.prompts.ValidationRules
import data.PreviousManifestData

object Copyright : TextPrompt {
    override val name: String = "Copyright"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = false
    )

    override val default: String? get() = PreviousManifestData.defaultLocaleManifest?.copyright

    override val extraText: String = "Example: Copyright (c) Microsoft Corporation"

    object Url : UrlPrompt {
        override val name: String = "Copyright url"

        override val previousUrl: io.ktor.http.Url? get() = PreviousManifestData.defaultLocaleManifest?.copyrightUrl

        override val description: String = "package's copyright url"
    }
}
