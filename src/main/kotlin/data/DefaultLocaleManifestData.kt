package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.Schema
import schemas.Schemas
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
    private val previousManifestData: PreviousManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()

    fun createDefaultLocaleManifest(): String {
        return getDefaultLocaleManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = if (::publisher.isInitialized) {
                publisher
            } else {
                previousManifestData.remoteDefaultLocaleData?.publisher
            } ?: "",
            publisherUrl = publisherUrl
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.publisherUrl,
            publisherSupportUrl = publisherSupportUrl
                .takeIf { it?.isNotBlank() == true }
                ?: previousManifestData.remoteDefaultLocaleData?.publisherSupportUrl,
            privacyUrl = publisherPrivacyUrl
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.privacyUrl,
            author = author.takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.author,
            packageName = sharedManifestData.packageName
                ?: previousManifestData.remoteDefaultLocaleData?.packageName
                ?: "",
            packageUrl = packageUrl?.ifBlank { null },
            license = when {
                ::license.isInitialized -> license
                else -> previousManifestData.remoteDefaultLocaleData?.license ?: ""
            },
            licenseUrl = licenseUrl
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.licenseUrl,
            copyright = copyright
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.copyright,
            copyrightUrl = copyrightUrl
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.copyrightUrl,
            shortDescription = when {
                ::shortDescription.isInitialized -> shortDescription
                else -> previousManifestData.remoteDefaultLocaleData?.shortDescription ?: ""
            },
            description = description
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.description,
            moniker = moniker
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.moniker,
            tags = tags.takeIf { it?.isNotEmpty() == true } ?: previousManifestData.remoteDefaultLocaleData?.tags,
            releaseNotesUrl = releaseNotesUrl
                .takeIf { it?.isNotBlank() == true } ?: previousManifestData.remoteDefaultLocaleData?.releaseNotesUrl,
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
