package data.locale

import commands.interfaces.UrlPrompt
import io.ktor.client.HttpClient
import io.ktor.http.Url

class PackageUrl(override val previousUrl: Url?, override val client: HttpClient) : UrlPrompt {
    override val name: String = "Package url"

    override val description: String = "package home page"
}
