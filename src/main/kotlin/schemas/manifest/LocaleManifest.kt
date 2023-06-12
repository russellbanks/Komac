package schemas.manifest

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import schemas.Schemas

/**
 * A representation of a multiple-file manifest representing app metadata in other locale in the OWC. v1.4.0
 */
@Serializable
data class LocaleManifest(
    val packageIdentifier: String,
    val packageVersion: String,
    val packageLocale: String,
    val publisher: String? = null,
    @Contextual val publisherUrl: Url? = null,
    @Contextual val publisherSupportUrl: Url? = null,
    @Contextual val privacyUrl: Url? = null,
    val author: String? = null,
    val packageName: String? = null,
    @Contextual val packageUrl: Url? = null,
    val license: String? = null,
    @Contextual val licenseUrl: Url? = null,
    val copyright: String? = null,
    @Contextual val copyrightUrl: Url? = null,
    val shortDescription: String? = null,
    val description: String? = null,
    val tags: List<String>? = null,
    val agreements: List<Agreement>? = null,
    val releaseNotes: String? = null,
    @Contextual val releaseNotesUrl: Url? = null,
    @Contextual val purchaseUrl: Url? = null,
    val installationNotes: String? = null,
    val documentations: List<Documentation>? = null,
    val manifestType: String,
    val manifestVersion: String
) : Manifest() {
    @Serializable
    data class Agreement(
        val agreementLabel: String? = null,
        val agreement: String? = null,
        @Contextual val agreementUrl: Url? = null
    )

    @Serializable
    data class Documentation(
        val documentLabel: String? = null,
        @Contextual val documentUrl: Url? = null
    )

    override fun toString() = Schemas.buildManifestString(
        manifest = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
