package schemas.manifest

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.Schema
import schemas.Schemas

/**
 * A representation of a multiple-file manifest representing app metadata in other locale in the OWC. v1.4.0
 */
@Serializable
data class LocaleManifest(
    @SerialName("PackageIdentifier") val packageIdentifier: String,
    @SerialName("PackageVersion") val packageVersion: String,
    @SerialName("PackageLocale") val packageLocale: String,
    @SerialName("Publisher") val publisher: String? = null,
    @SerialName("PublisherUrl") @Contextual val publisherUrl: Url? = null,
    @SerialName("PublisherSupportUrl") @Contextual val publisherSupportUrl: Url? = null,
    @SerialName("PrivacyUrl") @Contextual val privacyUrl: Url? = null,
    @SerialName("Author") val author: String? = null,
    @SerialName("PackageName") val packageName: String? = null,
    @SerialName("PackageUrl") @Contextual val packageUrl: Url? = null,
    @SerialName("License") val license: String? = null,
    @SerialName("LicenseUrl") @Contextual val licenseUrl: Url? = null,
    @SerialName("Copyright") val copyright: String? = null,
    @SerialName("CopyrightUrl") @Contextual val copyrightUrl: Url? = null,
    @SerialName("ShortDescription") val shortDescription: String? = null,
    @SerialName("Description") val description: String? = null,
    @SerialName("Tags") val tags: List<String>? = null,
    @SerialName("Agreements") val agreements: List<Agreement>? = null,
    @SerialName("ReleaseNotes") val releaseNotes: String? = null,
    @SerialName("ReleaseNotesUrl") @Contextual val releaseNotesUrl: Url? = null,
    @SerialName("PurchaseUrl") @Contextual val purchaseUrl: Url? = null,
    @SerialName("InstallationNotes") val installationNotes: String? = null,
    @SerialName("Documentations") val documentations: List<Documentation>? = null,
    @SerialName("ManifestType") val manifestType: String,
    @SerialName("ManifestVersion") val manifestVersion: String
) {
    @Serializable
    data class Agreement(
        @SerialName("AgreementLabel") val agreementLabel: String? = null,
        @SerialName("Agreement") val agreement: String? = null,
        @SerialName("AgreementUrl") @Contextual val agreementUrl: Url? = null
    )

    @Serializable
    data class Documentation(
        @SerialName("DocumentLabel") val documentLabel: String? = null,
        @SerialName("DocumentUrl") @Contextual val documentUrl: Url? = null
    )

    override fun toString() = Schemas().buildManifestString(
        schema = Schema.Locale,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
