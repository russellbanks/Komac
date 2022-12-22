package schemas

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * A representation of a multiple-file manifest representing app metadata in other locale in the OWC. v1.4.0
 */
@Serializable
data class LocaleManifest(
    @SerialName("PackageIdentifier") val packageIdentifier: String,
    @SerialName("PackageVersion") val packageVersion: String,
    @SerialName("PackageLocale") val packageLocale: String,
    @SerialName("Publisher") val publisher: String? = null,
    @SerialName("PublisherUrl") val publisherUrl: String? = null,
    @SerialName("PublisherSupportUrl") val publisherSupportUrl: String? = null,
    @SerialName("PrivacyUrl") val privacyUrl: String? = null,
    @SerialName("Author") val author: String? = null,
    @SerialName("PackageName") val packageName: String? = null,
    @SerialName("PackageUrl") val packageUrl: String? = null,
    @SerialName("License") val license: String? = null,
    @SerialName("LicenseUrl") val licenseUrl: String? = null,
    @SerialName("Copyright") val copyright: String? = null,
    @SerialName("CopyrightUrl") val copyrightUrl: String? = null,
    @SerialName("ShortDescription") val shortDescription: String? = null,
    @SerialName("Description") val description: String? = null,
    @SerialName("Tags") val tags: List<String>? = null,
    @SerialName("Agreements") val agreements: List<Agreement>? = null,
    @SerialName("ReleaseNotes") val releaseNotes: String? = null,
    @SerialName("ReleaseNotesUrl") val releaseNotesUrl: String? = null,
    @SerialName("PurchaseUrl") val purchaseUrl: String? = null,
    @SerialName("InstallationNotes") val installationNotes: String? = null,
    @SerialName("Documentations") val documentations: List<Documentation>? = null,
    @SerialName("ManifestType") val manifestType: String,
    @SerialName("ManifestVersion") val manifestVersion: String
) {
    @Serializable
    data class Agreement(
        @SerialName("AgreementLabel") val agreementLabel: String? = null,
        @SerialName("Agreement") val agreement: String? = null,
        @SerialName("AgreementUrl") val agreementUrl: String? = null
    )

    @Serializable
    data class Documentation(
        @SerialName("DocumentLabel") val documentLabel: String? = null,
        @SerialName("DocumentUrl") val documentUrl: String? = null
    )
}
