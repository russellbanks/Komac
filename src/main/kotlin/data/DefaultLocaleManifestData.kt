package data

import data.shared.Locale
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.DefaultLocaleManifest

object DefaultLocaleManifestData : KoinComponent {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val schemas: Schemas by inject()
    private val parameterLocaleMetadata = allManifestData.additionalMetadata?.locales?.find {
        it.name.equals(other = allManifestData.defaultLocale, ignoreCase = true)
    }

    suspend fun createDefaultLocaleManifest(): String = with(allManifestData) {
        val previousDefaultLocaleData = previousManifestData.remoteDefaultLocaleData.await()
        return getDefaultLocaleManifestBase().copy(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            packageLocale = defaultLocale
                ?: previousManifestData.remoteVersionData.await()?.defaultLocale
                ?: Locale.defaultLocale,
            publisher = publisher ?: previousDefaultLocaleData?.publisher ?: "",
            publisherUrl = (
                publisherUrl
                    ?: previousDefaultLocaleData?.publisherUrl
                    ?: gitHubDetection?.publisherUrl
                )?.ifBlank { null },
            publisherSupportUrl = previousDefaultLocaleData?.publisherSupportUrl
                ?: gitHubDetection?.publisherSupportUrl
                ?: pageScraper?.supportUrl?.await(),
            privacyUrl = previousDefaultLocaleData?.privacyUrl?.ifBlank { null }
                ?: gitHubDetection?.privacyUrl
                ?: pageScraper?.privacyUrl?.await(),
            author = author?.ifBlank { null } ?: previousDefaultLocaleData?.author,
            packageName = packageName ?: previousDefaultLocaleData?.packageName ?: "",
            packageUrl = packageUrl ?: previousDefaultLocaleData?.packageUrl ?: gitHubDetection?.packageUrl,
            license = license ?: gitHubDetection?.license ?: previousDefaultLocaleData?.license ?: "",
            licenseUrl = licenseUrl?.ifBlank { null }
                ?: previousDefaultLocaleData?.licenseUrl
                ?: gitHubDetection?.licenseUrl,
            copyright = copyright?.ifBlank { null } ?: previousDefaultLocaleData?.copyright,
            copyrightUrl = copyrightUrl?.ifBlank { null } ?: previousDefaultLocaleData?.copyrightUrl,
            shortDescription = when {
                shortDescription != null -> shortDescription!!
                else -> {
                    previousDefaultLocaleData?.shortDescription ?: gitHubDetection?.shortDescription ?: ""
                }
            },
            description = (description?.ifBlank { null } ?: previousDefaultLocaleData?.description)
                ?.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n")
                ?.trim(),
            moniker = moniker?.ifBlank { null } ?: previousDefaultLocaleData?.moniker,
            tags = tags?.ifEmpty { null } ?: previousDefaultLocaleData?.tags,
            releaseNotesUrl = releaseNotesUrl?.ifBlank { null }
                ?: gitHubDetection?.releaseNotesUrl
                ?: parameterLocaleMetadata?.releaseNotesUrl,
            releaseNotes = (gitHubDetection?.releaseNotes ?: parameterLocaleMetadata?.releaseNotes)?.trim(),
            documentations = if (previousDefaultLocaleData?.documentations == null) {
                listOfNotNull(
                    pageScraper?.faqUrl?.await()?.let {
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

    private suspend fun getDefaultLocaleManifestBase(): DefaultLocaleManifest = with(allManifestData) {
        return previousManifestData.remoteDefaultLocaleData.await() ?: DefaultLocaleManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            packageLocale = Locale.defaultLocale,
            publisher = publisher as String,
            packageName = packageName as String,
            license = license as String,
            shortDescription = shortDescription as String,
            manifestType = Schemas.defaultLocaleManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        )
    }
}
