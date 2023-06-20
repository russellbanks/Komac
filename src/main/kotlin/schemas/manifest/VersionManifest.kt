package schemas.manifest

import data.shared.Locale
import kotlinx.serialization.Serializable
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
) : Manifest() {
    override fun toString() = Schemas.buildManifestString(
        manifest = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )

    companion object {
        fun create(
            packageIdentifier: String,
            packageVersion: String,
            defaultLocale: String?,
            previousDefaultLocale: String?,
            manifestOverride: String? = null
        ): VersionManifest = VersionManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale
                ?: previousDefaultLocale
                ?: Locale.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        )

        /**
         * Returns the name of the YAML file containing the manifest for the given package identifier and version.
         *
         * @param identifier the package identifier to get the manifest name for
         * @return a string representing the name of the YAML file containing the manifest for the given identifier
         */
        fun getFileName(identifier: String) = "$identifier.yaml"
    }
}
