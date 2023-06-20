package data.locale

import io.menu.prompts.TextPrompt
import io.menu.prompts.UrlPrompt
import io.menu.prompts.ValidationRules
import schemas.manifest.DefaultLocaleManifest

class Copyright(private val previousDefaultLocaleManifest: DefaultLocaleManifest?) : TextPrompt {
    override val name: String = "Copyright"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = false
    )

    override val default: String? get() = previousDefaultLocaleManifest?.copyright

    override val extraText: String = "Example: Copyright (c) Microsoft Corporation"

    class Url(private val previousDefaultLocaleManifest: DefaultLocaleManifest?) : UrlPrompt {
        override val name: String = "Copyright url"

        override val previousUrl: io.ktor.http.Url? get() = previousDefaultLocaleManifest?.copyrightUrl

        override val description: String = "package's copyright url"
    }
}
