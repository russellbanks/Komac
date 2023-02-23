package data

import data.shared.Locale
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.DefaultLocaleManifest

@Single
class DefaultLocaleManifestData : KoinComponent {
    lateinit var license: String
    lateinit var shortDescription: String
    var moniker: String? = null
    var publisherUrl: Url? = null
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
    private val schemas: Schemas by inject()
    private val parameterLocaleMetadata = sharedManifestData.additionalMetadata?.locales?.find {
        it.name.equals(other = sharedManifestData.defaultLocale, ignoreCase = true)
    }

    suspend fun createDefaultLocaleManifest(): String {
        val previousDefaultLocaleData = previousManifestData.remoteDefaultLocaleData.await()
        return getDefaultLocaleManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = (sharedManifestData.defaultLocale
                ?: previousManifestData.remoteVersionData.await()?.defaultLocale)!!,
            publisher = sharedManifestData.publisher ?: previousDefaultLocaleData?.publisher ?: "",
            publisherUrl = (publisherUrl
                ?: previousDefaultLocaleData?.publisherUrl
                ?: sharedManifestData.gitHubDetection?.publisherUrl)?.ifBlank { null },
            publisherSupportUrl = previousDefaultLocaleData?.publisherSupportUrl
                ?: sharedManifestData.gitHubDetection?.publisherSupportUrl
                ?: sharedManifestData.pageScraper?.supportUrl?.await(),
            privacyUrl = previousDefaultLocaleData?.privacyUrl?.ifBlank { null }
                ?: sharedManifestData.gitHubDetection?.privacyUrl?.await()
                ?: sharedManifestData.pageScraper?.privacyUrl?.await(),
            author = author?.ifBlank { null } ?: previousDefaultLocaleData?.author,
            packageName = sharedManifestData.packageName
                ?: previousDefaultLocaleData?.packageName ?: "",
            packageUrl = packageUrl
                ?: previousDefaultLocaleData?.packageUrl
                ?: sharedManifestData.gitHubDetection?.packageUrl,
            license = when {
                ::license.isInitialized -> license
                else -> sharedManifestData.gitHubDetection?.license ?: previousDefaultLocaleData?.license ?: ""
            },
            licenseUrl = licenseUrl?.ifBlank { null }
                ?: previousDefaultLocaleData?.licenseUrl
                ?: sharedManifestData.gitHubDetection?.licenseUrl,
            copyright = copyright?.ifBlank { null } ?: previousDefaultLocaleData?.copyright,
            copyrightUrl = copyrightUrl?.ifBlank { null } ?: previousDefaultLocaleData?.copyrightUrl,
            shortDescription = when {
                ::shortDescription.isInitialized -> shortDescription
                else -> {
                    previousDefaultLocaleData?.shortDescription
                        ?: sharedManifestData.gitHubDetection?.shortDescription ?: ""
                }
            },
            description = (description?.ifBlank { null } ?: previousDefaultLocaleData?.description)
                ?.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n")
                ?.trim(),
            moniker = moniker?.ifBlank { null } ?: previousDefaultLocaleData?.moniker,
            tags = tags?.ifEmpty { null } ?: previousDefaultLocaleData?.tags,
            releaseNotesUrl = releaseNotesUrl?.ifBlank { null }
                ?: sharedManifestData.gitHubDetection?.releaseNotesUrl
                ?: parameterLocaleMetadata?.releaseNotesUrl,
            releaseNotes = (sharedManifestData.gitHubDetection?.releaseNotes
                ?: parameterLocaleMetadata?.releaseNotes)?.trim(),
            documentations = if (previousDefaultLocaleData?.documentations == null) {
                listOfNotNull(
                    sharedManifestData.pageScraper?.faqUrl?.await()?.let {
                        DefaultLocaleManifest.Documentation(documentLabel = "FAQ", documentUrl = it)
                    }
                ).ifEmpty { null }
            } else {
                previousDefaultLocaleData.documentations
            },
            manifestType = Schemas.defaultLocaleManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private inline fun Url.ifBlank(defaultValue: () -> Url?): Url? =
        if (this == Url(URLBuilder())) defaultValue() else this

    private suspend fun getDefaultLocaleManifestBase(): DefaultLocaleManifest {
        return previousManifestData.remoteDefaultLocaleData.await() ?: DefaultLocaleManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            packageLocale = Locale.defaultLocale,
            publisher = sharedManifestData.publisher!!,
            packageName = sharedManifestData.packageName!!,
            license = license,
            shortDescription = shortDescription,
            manifestType = Schemas.defaultLocaleManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        )
    }
}
