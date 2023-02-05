package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.EncodeConfig
import schemas.manifest.VersionManifest

@Single
class VersionManifestData : KoinComponent {
    private val schemasImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()

    fun createVersionManifest(): String {
        return VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = sharedManifestData.defaultLocale,
            manifestType = Schemas.versionManifestType,
            manifestVersion = schemasImpl.manifestOverride ?: Schemas.manifestVersion
        ).toEncodedYaml()
    }
    private fun VersionManifest.toEncodedYaml(): String {
        return Schemas.buildManifestString(
            schema = Schema.Version,
            rawString = EncodeConfig.yamlDefault.encodeToString(
                serializer = VersionManifest.serializer(),
                value = this@toEncodedYaml
            )
        )
    }
}
