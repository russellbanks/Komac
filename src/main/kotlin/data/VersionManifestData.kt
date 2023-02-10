package data

import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.VersionManifest

object VersionManifestData : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val previousVersionData = get<PreviousManifestData>().remoteVersionData

    suspend fun createVersionManifest(): String {
        return VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = (sharedManifestData.defaultLocale ?: previousVersionData.await()?.defaultLocale)!!,
            manifestType = Schemas.versionManifestType,
            manifestVersion = get<Schemas>().manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }
}
