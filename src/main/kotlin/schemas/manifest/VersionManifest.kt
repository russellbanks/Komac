package schemas.manifest

import kotlinx.serialization.Serializable
import schemas.Schema
import schemas.Schemas

/**
 * A representation of a multi-file manifest representing an app version in the OWC. v1.4.0
 */
@Serializable
data class VersionManifest(
    val packageIdentifier: String,
    val packageVersion: String,
    val defaultLocale: String,
    val manifestType: String,
    val manifestVersion: String
) {
    override fun toString() = Schemas.buildManifestString(
        schema = Schema.Version,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
