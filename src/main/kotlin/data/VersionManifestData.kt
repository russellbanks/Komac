package data

import data.shared.Locale
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.VersionManifest

object VersionManifestData : KoinComponent {
    private val allManifestData: AllManifestData by inject()
    private val previousVersionData = get<PreviousManifestData>().remoteVersionData

    suspend fun createVersionManifest(): String = with(allManifestData) {
        return VersionManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            defaultLocale = defaultLocale ?: previousVersionData.await()?.defaultLocale ?: Locale.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = get<Schemas>().manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }
}
