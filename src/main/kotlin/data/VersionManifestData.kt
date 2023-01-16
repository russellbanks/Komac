package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.VersionManifest

@Single
class VersionManifestData : KoinComponent {
    private val schemaImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()

    fun createVersionManifest(): String {
        return VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = sharedManifestData.defaultLocale,
            manifestType = schemaImpl.versionSchema.properties.manifestType.const,
            manifestVersion = schemaImpl.versionSchema.properties.manifestVersion.default,
        ).toEncodedYaml()
    }
    private fun VersionManifest.toEncodedYaml(): String {
        return Schemas.buildManifestString(
            schema = Schema.DefaultLocale,
            rawString = YamlConfig.default.encodeToString(
                serializer = VersionManifest.serializer(),
                value = this@toEncodedYaml
            )
        )
    }
}
