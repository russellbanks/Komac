package schemas.manifest

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import schemas.Schemas

/**
 * A representation of a multiple-file manifest representing a default app metadata in the OWC. v1.4.0
 */
@Serializable
data class DefaultLocaleManifest(
    val packageIdentifier: String,
    val packageVersion: String,
    val packageLocale: String,
    val publisher: String,
    @Contextual val publisherUrl: Url? = null,
    @Contextual val publisherSupportUrl: Url? = null,
    @Contextual val privacyUrl: Url? = null,
    val author: String? = null,
    val packageName: String,
    @Contextual val packageUrl: Url? = null,
    val license: String,
    @Contextual val licenseUrl: Url? = null,
    val copyright: String? = null,
    @Contextual val copyrightUrl: Url? = null,
    val shortDescription: String,
    val description: String? = null,
    val moniker: String? = null,
    val tags: List<String>? = null,
    val agreements: List<Agreement>? = null,
    val releaseNotes: String? = null,
    @Contextual val releaseNotesUrl: Url? = null,
    val purchaseUrl: String? = null,
    val installationNotes: String? = null,
    val documentations: List<Documentation>? = null,
    val manifestType: String,
    val manifestVersion: String
) : Schema() {
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
        schema = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
