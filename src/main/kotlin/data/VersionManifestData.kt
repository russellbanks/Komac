package data

import data.shared.Locale
import schemas.Schemas
import schemas.manifest.VersionManifest

object VersionManifestData {
    fun createVersionManifest(manifestOverride: String? = null): VersionManifest = with(ManifestData) {
        return VersionManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale
                ?: PreviousManifestData.versionManifest?.defaultLocale
                ?: Locale.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        )
    }
}
