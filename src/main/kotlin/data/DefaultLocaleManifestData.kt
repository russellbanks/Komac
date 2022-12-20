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
    lateinit var license: String
    lateinit var shortDescription: String
    var moniker: String? = null
    var publisherUrl: String? = null
    var publisherSupportUrl: String? = null
    var publisherPrivacyUrl: String? = null
    var author: String? = null
    var packageUrl: String? = null
    var licenseUrl: String? = null
    var copyright: String? = null
    var copyrightUrl: String? = null
    var tags: List<String>? = null
    var description: String? = null
    var releaseNotesUrl: String? = null

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
            publisherSupportUrl = publisherSupportUrl?.ifBlank { null },
            privacyUrl = publisherPrivacyUrl?.ifBlank { null },
            author = author?.ifBlank { null },
            packageName = packageName,
            packageUrl = packageUrl?.ifBlank { null },
            license = license,
            licenseUrl = licenseUrl?.ifBlank { null },
            copyright = copyright?.ifBlank { null },
            copyrightUrl = copyrightUrl?.ifBlank { null },
            shortDescription = shortDescription,
            description = description?.ifBlank { null },
            moniker = moniker?.ifBlank { null },
            tags = tags?.ifEmpty { null },
            releaseNotesUrl = releaseNotesUrl?.ifBlank { null },
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
