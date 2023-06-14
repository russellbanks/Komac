package data.locale

import io.ktor.http.Url
import io.menu.prompts.UrlPrompt
import schemas.manifest.DefaultLocaleManifest

class PackageUrl(private val defaultLocaleManifest: DefaultLocaleManifest?) : UrlPrompt {
    override val name: String = "Package url"

    override val previousUrl: Url? get() = defaultLocaleManifest?.packageUrl

    override val description: String = "package home page"
}
