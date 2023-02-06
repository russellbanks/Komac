package schemas.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.Schema
import schemas.Schemas

/**
 * A representation of a multi-file manifest representing an app version in the OWC. v1.4.0
 */
@Serializable
data class VersionManifest(
    @SerialName("PackageIdentifier") val packageIdentifier: String,
    @SerialName("PackageVersion") val packageVersion: String,
    @SerialName("DefaultLocale") val defaultLocale: String,
    @SerialName("ManifestType") val manifestType: String,
    @SerialName("ManifestVersion") val manifestVersion: String
) {
    override fun toString() = Schemas().buildManifestString(
        schema = Schema.Version,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
