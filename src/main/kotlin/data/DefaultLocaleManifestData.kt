package data

import data.locale.Tags
import data.shared.Locale
import io.ktor.http.Url
import schemas.Schemas
import schemas.manifest.DefaultLocaleManifest

object DefaultLocaleManifestData {
    suspend fun createDefaultLocaleManifest(manifestOverride: String? = null): String = with(ManifestData) {
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
            publisherUrl = (publisherUrl ?: previousDefaultLocaleData?.publisherUrl ?: gitHubDetection?.publisherUrl)
                ?.ifBlank { null },
            publisherSupportUrl = previousDefaultLocaleData?.publisherSupportUrl
                ?: gitHubDetection?.publisherSupportUrl
                ?: pageScraper?.supportUrl?.await(),
            privacyUrl = (previousDefaultLocaleData?.privacyUrl
                ?: gitHubDetection?.privacyUrl
                ?: pageScraper?.privacyUrl?.await())
                ?.ifBlank { null },
            author = (author ?: previousDefaultLocaleData?.author)?.ifBlank { null },
            packageName = packageName ?: previousDefaultLocaleData?.packageName.orEmpty(),
            packageUrl = packageUrl ?: previousDefaultLocaleData?.packageUrl ?: gitHubDetection?.packageUrl,
            license = license ?: gitHubDetection?.license ?: previousDefaultLocaleData?.license.orEmpty(),
            licenseUrl = (licenseUrl ?: previousDefaultLocaleData?.licenseUrl)?.ifBlank { null }
                ?: gitHubDetection?.licenseUrl,
            copyright = (copyright ?: previousDefaultLocaleData?.copyright)?.ifBlank { null },
            copyrightUrl = (copyrightUrl ?: previousDefaultLocaleData?.copyrightUrl)?.ifBlank { null },
            shortDescription = shortDescription
                ?: previousDefaultLocaleData?.shortDescription
                ?: gitHubDetection?.shortDescription.orEmpty(),
            description = (description ?: previousDefaultLocaleData?.description)
                ?.replace(Regex("([A-Z][a-z].*?[.!?]) ?(?=\$|[A-Z])"), "$1\n")
                ?.lines()
                ?.joinToString("\n") { it.trim() }
                ?.trim()
                ?.ifBlank { null },
            moniker = (moniker ?: previousDefaultLocaleData?.moniker)?.ifBlank { null },
            tags = (tags ?: previousDefaultLocaleData?.tags)?.take(Tags.validationRules.maxItems)?.ifEmpty { null },
            releaseNotesUrl = (releaseNotesUrl
                ?: gitHubDetection?.releaseNotesUrl
                ?: parameterLocaleMetadata?.releaseNotesUrl)
                ?.ifBlank { null },
            releaseNotes = (gitHubDetection?.releaseNotes ?: parameterLocaleMetadata?.releaseNotes)?.trim(),
            documentations = if (previousDefaultLocaleData?.documentations == null) {
                listOfNotNull(
                    pageScraper?.faqUrl?.await()?.let {
                        DefaultLocaleManifest.Documentation(documentLabel = "FAQ", documentUrl = it)
                    }
                )
            } else {
                previousDefaultLocaleData.documentations
            }.ifEmpty { null },
            manifestType = Schemas.defaultLocaleManifestType,
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private inline fun Url.ifBlank(defaultValue: () -> Url?): Url? = if (this == Url("")) defaultValue() else this

    private val defaultLocaleManifestBase: DefaultLocaleManifest
        get() = with(ManifestData) {
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
