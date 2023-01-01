package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.VersionManifest

@Single
class VersionManifestData : KoinComponent {
    private val schemaImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val versionSchema
        get() = schemaImpl.versionSchema

    fun createVersionManifest(): String {
        return VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = sharedManifestData.defaultLocale,
            manifestType = versionSchema.properties.manifestType.const,
            manifestVersion = versionSchema.properties.manifestVersion.default,
        ).let {
            get<GitHubImpl>().buildManifestString(get<SchemasImpl>().versionSchema.id) {
                appendLine(YamlConfig.defaultWithLocalDataSerializer.encodeToString(VersionManifest.serializer(), it))
            }
        }
    }
}
