package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.SchemasImpl

@Single
class DefaultLocaleManifestData : KoinComponent {
    lateinit var publisher: String
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

    private val sharedManifestData: SharedManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val defaultLocaleSchema
        get() = schemasImpl.defaultLocaleSchema

    fun createDefaultLocaleManifest(): String {
        return DefaultLocaleManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = publisher,
            publisherUrl = publisherUrl?.ifBlank { null },
            publisherSupportUrl = publisherSupportUrl?.ifBlank { null },
            privacyUrl = publisherPrivacyUrl?.ifBlank { null },
            author = author?.ifBlank { null },
            packageName = sharedManifestData.packageName,
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
        ).let {
            get<GitHubImpl>().buildManifestString(get<SchemasImpl>().defaultLocaleSchema.id) {
                appendLine(
                    YamlConfig.defaultWithLocalDataSerializer.encodeToString(DefaultLocaleManifest.serializer(), it)
                )
            }
        }
    }
}
