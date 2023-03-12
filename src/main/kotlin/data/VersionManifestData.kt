package data

import data.shared.Locale
import schemas.Schemas
import schemas.manifest.VersionManifest

object VersionManifestData {
    fun createVersionManifest(
        allManifestData: AllManifestData,
        manifestOverride: String? = null,
        previousVersionData: VersionManifest?
    ): String = with(allManifestData) {
        return VersionManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale ?: previousVersionData?.defaultLocale ?: Locale.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }
}
