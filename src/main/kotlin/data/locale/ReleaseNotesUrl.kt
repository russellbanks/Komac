package data.locale

import commands.interfaces.UrlPrompt
import io.ktor.client.HttpClient

class ReleaseNotesUrl(override val client: HttpClient) : UrlPrompt {
    override val name: String = "Release notes url"

    override val description: String = "package release notes url"
}
