package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.UrlPrompt
import commands.interfaces.ValidationRules
import io.ktor.client.HttpClient

class Publisher(previousPublisher: String?) : TextPrompt {

    override val name: String = "Publisher"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 256,
        minLength = 2,
        isRequired = true
    )

    override val default: String? = previousPublisher

    override val extraText: String = "Example: Microsoft Corporation"

    class Url(override val previousUrl: io.ktor.http.Url?, override val client: HttpClient) : UrlPrompt {
        override val name: String = "Publisher url"

        override val description: String = "publisher home page"
    }
}
