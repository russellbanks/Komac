package data

import data.shared.Locale
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import schemas.Schemas
import schemas.manifest.DefaultLocaleManifest

object DefaultLocaleManifestData {
    suspend fun createDefaultLocaleManifest(manifestOverride: String? = null): String = with(AllManifestData) {
        val parameterLocaleMetadata = additionalMetadata?.locales?.find {
            it.name.equals(other = defaultLocale, ignoreCase = true)
        }
        val previousDefaultLocaleData = PreviousManifestData.defaultLocaleManifest
        return defaultLocaleManifestBase.copy(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            packageLocale = defaultLocale
                ?: PreviousManifestData.versionManifest?.defaultLocale
                ?: Locale.defaultLocale,
            publisher = publisher ?: previousDefaultLocaleData?.publisher.orEmpty(),
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
            packageName = packageName ?: previousDefaultLocaleData?.packageName.orEmpty(),
            packageUrl = packageUrl ?: previousDefaultLocaleData?.packageUrl ?: gitHubDetection?.packageUrl,
            license = license ?: gitHubDetection?.license ?: previousDefaultLocaleData?.license.orEmpty(),
            licenseUrl = licenseUrl?.ifBlank { null }
                ?: previousDefaultLocaleData?.licenseUrl
                ?: gitHubDetection?.licenseUrl,
            copyright = copyright?.ifBlank { null } ?: previousDefaultLocaleData?.copyright,
            copyrightUrl = copyrightUrl?.ifBlank { null } ?: previousDefaultLocaleData?.copyrightUrl,
            shortDescription = shortDescription
                ?: previousDefaultLocaleData?.shortDescription
                ?: gitHubDetection?.shortDescription.orEmpty(),
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
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private inline fun Url.ifBlank(defaultValue: () -> Url?): Url? =
        if (this == Url(URLBuilder())) defaultValue() else this

    private val defaultLocaleManifestBase: DefaultLocaleManifest
        get() = with(AllManifestData) {
            return PreviousManifestData.defaultLocaleManifest ?: DefaultLocaleManifest(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                packageLocale = Locale.defaultLocale,
                publisher = publisher as String,
                packageName = packageName as String,
                license = license as String,
                shortDescription = shortDescription as String,
                manifestType = Schemas.defaultLocaleManifestType,
                manifestVersion = Schemas.manifestVersion
            )
        }
}
