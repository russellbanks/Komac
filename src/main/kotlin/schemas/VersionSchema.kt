package schemas

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class VersionSchema(
    @SerialName("\$id") val id: String,
    @SerialName("\$schema") val schema: String,
    @SerialName("description") val description: String,
    @SerialName("type") val type: String,
    @SerialName("properties") val properties: Properties,
    @SerialName("required") val required: List<String>
) {
    @Serializable
    data class Properties(
        @SerialName("PackageIdentifier") val packageIdentifier: PackageIdentifier,
        @SerialName("PackageVersion") val packageVersion: PackageVersion,
        @SerialName("DefaultLocale") val defaultLocale: DefaultLocale,
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
        data class DefaultLocale(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

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
