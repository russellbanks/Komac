package data

import io.ktor.http.Url
import network.HttpUtils.decodeHex
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.YamlConfig

@Single
class DefaultLocaleManifestData : KoinComponent {
    lateinit var license: String
    lateinit var shortDescription: String
    var moniker: String? = null
    var publisherUrl: Url? = null
    var publisherSupportUrl: Url? = null
    var publisherPrivacyUrl: Url? = null
    var author: String? = null
    var packageUrl: Url? = null
    var licenseUrl: Url? = null
    var copyright: String? = null
    var copyrightUrl: Url? = null
    var tags: List<String>? = null
    var description: String? = null
    var releaseNotesUrl: Url? = null

    private val sharedManifestData: SharedManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()

    suspend fun createDefaultLocaleManifest(): String {
        return getDefaultLocaleManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = sharedManifestData.publisher ?: previousManifestData.remoteDefaultLocaleData?.publisher ?: "",
            publisherUrl = (publisherUrl
                ?: previousManifestData.remoteDefaultLocaleData?.publisherUrl
                ?: sharedManifestData.gitHubDetection?.publisherUrl?.await())
                ?.decodeHex(),
            publisherSupportUrl = (publisherSupportUrl
                ?: previousManifestData.remoteDefaultLocaleData?.publisherSupportUrl
                ?: sharedManifestData.gitHubDetection?.publisherSupportUrl?.await())
                ?.decodeHex(),
            privacyUrl = (publisherPrivacyUrl
                ?: previousManifestData.remoteDefaultLocaleData?.privacyUrl
                ?: sharedManifestData.gitHubDetection?.privacyUrl?.await())
                ?.decodeHex(),
            author = author?.ifEmpty { null } ?: previousManifestData.remoteDefaultLocaleData?.author,
            packageName = sharedManifestData.packageName
                ?: previousManifestData.remoteDefaultLocaleData?.packageName ?: "",
            packageUrl = (packageUrl
                ?: previousManifestData.remoteDefaultLocaleData?.packageUrl
                ?: sharedManifestData.gitHubDetection?.packageUrl?.await())
                ?.decodeHex(),
            license = when {
                ::license.isInitialized -> license
                else -> sharedManifestData.gitHubDetection?.license?.await()
                    ?: previousManifestData.remoteDefaultLocaleData?.license ?: ""
            },
            licenseUrl = (licenseUrl
                ?: previousManifestData.remoteDefaultLocaleData?.licenseUrl
                ?: sharedManifestData.gitHubDetection?.licenseUrl?.await())
                ?.decodeHex(),
            copyright = copyright?.ifEmpty { null } ?: previousManifestData.remoteDefaultLocaleData?.copyright,
            copyrightUrl = (copyrightUrl ?: previousManifestData.remoteDefaultLocaleData?.copyrightUrl)?.decodeHex(),
            shortDescription = when {
                ::shortDescription.isInitialized -> shortDescription
                else -> {
                    previousManifestData.remoteDefaultLocaleData?.shortDescription
                        ?: sharedManifestData.gitHubDetection?.shortDescription?.await() ?: ""
                }
            },
            description = (description?.ifEmpty { null } ?: previousManifestData.remoteDefaultLocaleData?.description)
                ?.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n")
                ?.trim(),
            moniker = moniker?.ifEmpty { null } ?: previousManifestData.remoteDefaultLocaleData?.moniker,
            tags = tags?.ifEmpty { null } ?: previousManifestData.remoteDefaultLocaleData?.tags,
            releaseNotesUrl = (releaseNotesUrl
                ?: sharedManifestData.gitHubDetection?.releaseNotesUrl?.await())
                ?.decodeHex(),
            releaseNotes = sharedManifestData.gitHubDetection?.releaseNotes?.await()?.trim(),
            manifestType = schemasImpl.defaultLocaleSchema.properties.manifestType.const,
            manifestVersion = schemasImpl.manifestOverride ?: Schemas.manifestVersion
        ).toEncodedYaml()
    }

    private fun getDefaultLocaleManifestBase(): DefaultLocaleManifest {
        return previousManifestData.remoteDefaultLocaleData ?: DefaultLocaleManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = sharedManifestData.defaultLocale,
            publisher = sharedManifestData.publisher!!,
            packageName = sharedManifestData.packageName!!,
            license = license,
            shortDescription = shortDescription,
            manifestType = schemasImpl.defaultLocaleSchema.properties.manifestType.const,
            manifestVersion = Schemas.manifestVersion
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
