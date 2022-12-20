package data

import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.Schemas
import schemas.SchemasImpl
import schemas.TerminalInstance

@Single
class DefaultLocaleManifestData : KoinComponent {
    lateinit var publisher: String
    lateinit var packageName: String
    var moniker: String? = null
    var publisherUrl: String? = null
    var publisherSupportUrl: String? = null

    private val terminalInstance: TerminalInstance by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val defaultLocaleSchema
        get() = schemasImpl.defaultLocaleSchema

    fun createDefaultLocaleManifest() {
        DefaultLocaleManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = publisher,
            publisherUrl = publisherUrl?.ifBlank { null },
            packageName = packageName,
            license = "License",
            shortDescription = "ShortDescription",
            moniker = moniker?.ifBlank { null },
            manifestType = defaultLocaleSchema.properties.manifestType.const,
            manifestVersion = defaultLocaleSchema.properties.manifestVersion.default,
        ).also {
            Yaml(
                configuration = YamlConfiguration(
                    encodeDefaults = false,
                    singleLineStringStyle = SingleLineStringStyle.Plain
                )
            ).run {
                buildString {
                    appendLine(Schemas.Comments.createdBy)
                    appendLine(Schemas.Comments.languageServer(defaultLocaleSchema.id))
                    appendLine()
                    appendLine(encodeToString(DefaultLocaleManifest.serializer(), it))
                }.let(terminalInstance.terminal::print)
            }
        }
    }
}
