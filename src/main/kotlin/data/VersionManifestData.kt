package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.SchemasImpl
import schemas.TerminalInstance
import schemas.VersionManifest

@Single
class VersionManifestData : KoinComponent {
    private val terminalInstance: TerminalInstance by inject()
    private val schemaImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val versionSchema
        get() = schemaImpl.versionSchema

    fun createVersionManifest() {
        VersionManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            defaultLocale = sharedManifestData.defaultLocale,
            manifestType = versionSchema.properties.manifestType.const,
            manifestVersion = versionSchema.properties.manifestVersion.default,
        ).also {
            YamlConfig.default.run {
                buildString {
                    appendLine(Schemas.Comments.createdBy)
                    appendLine(Schemas.Comments.languageServer(versionSchema.id))
                    appendLine()
                    appendLine(encodeToString(VersionManifest.serializer(), it))
                }.let(terminalInstance.terminal::print)
            }
        }
    }
}
