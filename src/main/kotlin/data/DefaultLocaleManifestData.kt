package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.DefaultLocaleManifest

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
    private val previousManifestData: PreviousManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()

    suspend fun createDefaultLocaleManifest(): String {
        return getDefaultLocaleManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = when {
                ::publisher.isInitialized -> publisher
                else -> previousManifestData.remoteDefaultLocaleData?.publisher ?: ""
            },
            publisherUrl = publisherUrl.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.publisherUrl
                ?: sharedManifestData.gitHubDetection?.publisherUrl?.await(),
            publisherSupportUrl = publisherSupportUrl.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.publisherSupportUrl
                ?: sharedManifestData.gitHubDetection?.publisherSupportUrl?.await(),
            privacyUrl = publisherPrivacyUrl.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.privacyUrl
                ?: sharedManifestData.gitHubDetection?.privacyUrl?.await(),
            author = author.takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.author,
            packageName = sharedManifestData.packageName
                ?: previousManifestData.remoteDefaultLocaleData?.packageName ?: "",
            packageUrl = packageUrl?.ifBlank { null } ?: sharedManifestData.gitHubDetection?.packageUrl?.await(),
            license = when {
                ::license.isInitialized -> license
                else -> sharedManifestData.gitHubDetection?.license?.await()
                    ?: previousManifestData.remoteDefaultLocaleData?.license ?: ""
            },
            licenseUrl = licenseUrl.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.licenseUrl
                ?: sharedManifestData.gitHubDetection?.licenseUrl?.await(),
            copyright = copyright.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.copyright,
            copyrightUrl = copyrightUrl.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.copyrightUrl,
            shortDescription = when {
                ::shortDescription.isInitialized -> shortDescription
                else -> {
                    previousManifestData.remoteDefaultLocaleData?.shortDescription
                        ?: sharedManifestData.gitHubDetection?.shortDescription?.await() ?: ""
                }
            },
            description = (description
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.description)
                ?.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n")
                ?.trim(),
            moniker = moniker.takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.moniker,
            tags = tags.takeIf { it?.isNotEmpty() == true } ?: previousManifestData.remoteDefaultLocaleData?.tags,
            releaseNotesUrl = releaseNotesUrl.takeIf { it?.isNotBlank() == true }
                ?: sharedManifestData.gitHubDetection?.releaseNotesUrl?.await(),
            releaseNotes = sharedManifestData.gitHubDetection?.releaseNotes?.await()?.trim(),
            manifestType = schemasImpl.defaultLocaleSchema.properties.manifestType.const,
            manifestVersion = schemasImpl.defaultLocaleSchema.properties.manifestVersion.default
        ).toEncodedYaml()
    }

    private fun getDefaultLocaleManifestBase(): DefaultLocaleManifest {
        return previousManifestData.remoteDefaultLocaleData ?: DefaultLocaleManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = publisher,
            packageName = sharedManifestData.packageName!!,
            license = license,
            shortDescription = shortDescription,
            manifestType = schemasImpl.defaultLocaleSchema.properties.manifestType.const,
            manifestVersion = schemasImpl.defaultLocaleSchema.properties.manifestVersion.default
        )
    }

    private fun DefaultLocaleManifest.toEncodedYaml(): String {
        return Schemas.buildManifestString(
            schema = Schema.DefaultLocale,
            rawString = YamlConfig.default.encodeToString(
                serializer = DefaultLocaleManifest.serializer(),
                value = this@toEncodedYaml
            )
        )
    }
}
