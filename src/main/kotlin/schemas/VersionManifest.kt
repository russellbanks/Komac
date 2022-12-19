package schemas

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
)
