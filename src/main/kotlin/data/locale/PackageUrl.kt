package data.locale

import commands.interfaces.UrlPrompt
import data.PreviousManifestData
import io.ktor.http.Url

object PackageUrl : UrlPrompt {
    override val name: String = "Package url"

    override val previousUrl: Url? get() = PreviousManifestData.defaultLocaleManifest?.packageUrl

    override val description: String = "package home page"
}
