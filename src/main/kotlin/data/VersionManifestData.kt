package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.VersionManifest

@Single
class VersionManifestData : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()

    fun createVersionManifest(): String {
        return VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = sharedManifestData.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = get<Schemas>().manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }
}
