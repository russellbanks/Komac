package data.locale

import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import commands.interfaces.ValidationRules
import io.ktor.client.HttpClient

class License(previousLicense: String?) : TextPrompt {
    override val name = "License"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 512,
        minLength = 3,
        isRequired = true
    )

    override val default: String? = previousLicense

    override val extraText: String = "Example: MIT, GPL-3.0, Freeware, Proprietary"

    class Url(override val previousUrl: io.ktor.http.Url?, override val client: HttpClient) : UrlPrompt {
        override val name: String = "License url"

        override val description: String = "license page url"
    }
}
