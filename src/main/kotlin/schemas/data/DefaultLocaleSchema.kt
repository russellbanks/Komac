package schemas.data

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class DefaultLocaleSchema(
    @SerialName("\$id") val id: String,
    @SerialName("\$schema") val schema: String,
    @SerialName("description") val description: String,
    @SerialName("definitions") val definitions: Definitions,
    @SerialName("type") val type: String,
    @SerialName("properties") val properties: Properties,
    @SerialName("required") val required: List<String>
) : RemoteSchema {
    @Serializable
    data class Definitions(
        @SerialName("Url") val url: Url,
        @SerialName("Tag") val tag: Tag,
        @SerialName("Agreement") val agreement: Agreement,
        @SerialName("Documentation") val documentation: Documentation
    ) {
        @Serializable
        data class Url(
            @SerialName("type") val type: List<String>,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Tag(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Agreement(
            @SerialName("type") val type: String,
            @SerialName("properties") val properties: Properties
        ) {
            @Serializable
            data class Properties(
                @SerialName("AgreementLabel") val agreementLabel: AgreementLabel,
                @SerialName("Agreement") val agreement: Agreement,
                @SerialName("AgreementUrl") val agreementUrl: AgreementUrl
            ) {
                @Serializable
                data class AgreementLabel(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Agreement(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class AgreementUrl(
                    @SerialName("\$ref") val ref: String,
                    @SerialName("description") val description: String
                )
            }
        }

        @Serializable
        data class Documentation(
            @SerialName("type") val type: String,
            @SerialName("properties") val properties: Properties
        ) {
            @Serializable
            data class Properties(
                @SerialName("DocumentLabel") val documentLabel: DocumentLabel,
                @SerialName("DocumentUrl") val documentUrl: DocumentUrl
            ) {
                @Serializable
                data class DocumentLabel(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class DocumentUrl(
                    @SerialName("\$ref") val ref: String,
                    @SerialName("description") val description: String
                )
            }
        }
    }

    @Serializable
    data class Properties(
        @SerialName("PackageIdentifier") val packageIdentifier: PackageIdentifier,
        @SerialName("PackageVersion") val packageVersion: PackageVersion,
        @SerialName("PackageLocale") val packageLocale: PackageLocale,
        @SerialName("Publisher") val publisher: Publisher,
        @SerialName("PublisherUrl") val publisherUrl: PublisherUrl,
        @SerialName("PublisherSupportUrl") val publisherSupportUrl: PublisherSupportUrl,
        @SerialName("PrivacyUrl") val privacyUrl: PrivacyUrl,
        @SerialName("Author") val author: Author,
        @SerialName("PackageName") val packageName: PackageName,
        @SerialName("PackageUrl") val packageUrl: PackageUrl,
        @SerialName("License") val license: License,
        @SerialName("LicenseUrl") val licenseUrl: LicenseUrl,
        @SerialName("Copyright") val copyright: Copyright,
        @SerialName("CopyrightUrl") val copyrightUrl: CopyrightUrl,
        @SerialName("ShortDescription") val shortDescription: ShortDescription,
        @SerialName("Description") val description: Description,
        @SerialName("Moniker") val moniker: Moniker,
        @SerialName("Tags") val tags: Tags,
        @SerialName("Agreements") val agreements: Agreements,
        @SerialName("ReleaseNotes") val releaseNotes: ReleaseNotes,
        @SerialName("ReleaseNotesUrl") val releaseNotesUrl: ReleaseNotesUrl,
        @SerialName("PurchaseUrl") val purchaseUrl: PurchaseUrl,
        @SerialName("InstallationNotes") val installationNotes: InstallationNotes,
        @SerialName("Documentations") val documentations: Documentations,
        @SerialName("ManifestType") val manifestType: ManifestType,
        @SerialName("ManifestVersion") val manifestVersion: ManifestVersion
    ) {
        @Serializable
        data class PackageIdentifier(
            @SerialName("type") val type: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PackageVersion(
            @SerialName("type") val type: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PackageLocale(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Publisher(
            @SerialName("type") val type: String,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PublisherUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PublisherSupportUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PrivacyUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Author(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PackageName(
            @SerialName("type") val type: String,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PackageUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class License(
            @SerialName("type") val type: String,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class LicenseUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Copyright(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class CopyrightUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ShortDescription(
            @SerialName("type") val type: String,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Description(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Moniker(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Tags(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class Agreements(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class ReleaseNotes(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ReleaseNotesUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PurchaseUrl(
            @SerialName("\$ref") val ref: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class InstallationNotes(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Documentations(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class ManifestType(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("const") val const: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ManifestVersion(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("description") val description: String
        )
    }
}
