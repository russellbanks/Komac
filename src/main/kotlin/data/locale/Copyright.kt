package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import commands.interfaces.ValidationRules
import io.ktor.client.HttpClient

class Copyright(previousCopyright: String?) : TextPrompt {
    override val name: String = "Copyright"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = false
    )

    override val default: String? = previousCopyright

    override val extraText: String = "Example: Copyright (c) Microsoft Corporation"

    class Url(override val previousUrl: io.ktor.http.Url?, override val client: HttpClient) : UrlPrompt {
        override val name: String = "Copyright url"

        override val description: String = "package's copyright url"
    }
}
