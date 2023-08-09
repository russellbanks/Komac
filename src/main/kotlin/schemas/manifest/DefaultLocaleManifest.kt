package schemas.manifest

import data.PreviousManifestData
import data.locale.Tags
import data.shared.Locale
import github.GitHubDetection
import github.ReleaseNotesFormatter
import github.ReleaseNotesFormatter.cutToCharLimitWithLines
import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import network.WebPageScraper
import schemas.AdditionalMetadata
import schemas.SchemaType
import schemas.Schemas

/**
 * A representation of a multiple-file manifest representing a default app metadata in the OWC. v1.5.0
 */
@Suppress("unused")
@Serializable
data class DefaultLocaleManifest(
    val packageIdentifier: String,
    val packageVersion: String,
    val packageLocale: String,
    val publisher: String,
    @Contextual val publisherUrl: Url? = null,
    @Contextual val publisherSupportUrl: Url? = null,
    @Contextual val privacyUrl: Url? = null,
    val author: String? = null,
    val packageName: String,
    @Contextual val packageUrl: Url? = null,
    val license: String,
    @Contextual val licenseUrl: Url? = null,
    val copyright: String? = null,
    @Contextual val copyrightUrl: Url? = null,
    val shortDescription: String,
    val description: String? = null,
    val moniker: String? = null,
    val tags: List<String>? = null,
    val agreements: List<Agreement>? = null,
    val releaseNotes: String? = null,
    @Contextual val releaseNotesUrl: Url? = null,
    val purchaseUrl: String? = null,
    val installationNotes: String? = null,
    val documentations: List<Documentation>? = null,
    val icons: List<Icon>? = null,
    val manifestType: String,
    val manifestVersion: String = Schemas.MANIFEST_VERSION
) : Manifest() {
    @Serializable
    data class Agreement(
        val agreementLabel: String? = null,
        val agreement: String? = null,
        @Contextual val agreementUrl: Url? = null
    )

    @Serializable
    data class Documentation(
        val documentLabel: String? = null,
        @Contextual val documentUrl: Url? = null
    )

    @Serializable
    data class Icon(
        @Contextual val iconUrl: Url,
        val iconFileType: IconFileType?,
        val iconResolution: IconResolution? = null,
        val iconTheme: IconTheme? = null,
        val iconSha256: String? = null
    ) {
        enum class IconFileType {
            PNG,
            JPEG,
            ICO
        }

        enum class IconResolution {
            CUSTOM,
            @SerialName("16x16") SIZE16,
            @SerialName("20x20") SIZE20,
            @SerialName("24x24") SIZE24,
            @SerialName("30x30") SIZE30,
            @SerialName("32x32") SIZE32,
            @SerialName("36x36") SIZE36,
            @SerialName("40x40") SIZE40,
            @SerialName("48x48") SIZE48,
            @SerialName("60x60") SIZE60,
            @SerialName("64x64") SIZE64,
            @SerialName("72x72") SIZE72,
            @SerialName("80x80") SIZE80,
            @SerialName("96x96") SIZE96,
            @SerialName("256x256") SIZE256
        }

        enum class IconTheme {
            DEFAULT,
            LIGHT,
            DARK,
            HIGH_CONTRAST
        }
    }

    override fun toString() = Schemas.buildManifestString(
        manifest = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )

    companion object {
        /**
         * Returns the name of the YAML file containing the localized manifest for the given package identifier and default
         * locale.
         *
         * @param identifier the package identifier to get the manifest name for
         * @param defaultLocale the default locale to get the manifest name for, if available
         * @param previousDefaultLocale the previously set default locale, if any
         * @return a string representing the name of the YAML file containing the localized manifest for the given
         * identifier and default locale
         */
        fun getFileName(
            identifier: String,
            defaultLocale: String? = null,
            previousDefaultLocale: String?
        ) = "$identifier.locale.${defaultLocale ?: previousDefaultLocale ?: Locale.DEFAULT_LOCALE}.yaml"

        private fun getBase(
            packageIdentifier: String,
            packageVersion: String,
            publisher: String,
            packageName: String,
            license: String,
            shortDescription: String,
            previousDefaultLocaleManifest: DefaultLocaleManifest?
        ): DefaultLocaleManifest = previousDefaultLocaleManifest ?: DefaultLocaleManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            packageLocale = Locale.DEFAULT_LOCALE,
            publisher = publisher,
            packageName = packageName,
            license = license,
            shortDescription = shortDescription,
            manifestType = SchemaType.DEFAULT_LOCALE,
            manifestVersion = Schemas.MANIFEST_VERSION
        )

        suspend fun create(
            packageIdentifier: String,
            packageVersion: String,
            defaultLocale: String? = null,
            license: String,
            licenseUrl: Url? = null,
            author: String? = null,
            packageName: String,
            publisher: String,
            publisherUrl: Url? = null,
            packageUrl: Url? = null,
            copyright: String? = null,
            copyrightUrl: Url? = null,
            shortDescription: String,
            moniker: String? = null,
            tags: List<String>? = null,
            releaseNotesUrl: Url? = null,
            manifestOverride: String? = null,
            description: String? = null,
            pageScraper: WebPageScraper?,
            gitHubDetection: GitHubDetection?,
            additionalMetadata: AdditionalMetadata? = null,
            previousManifestData: PreviousManifestData?
        ): DefaultLocaleManifest {
            val parameterLocaleMetadata = additionalMetadata?.locales?.find {
                it.name.equals(other = defaultLocale, ignoreCase = true)
            }
            val previousDefaultLocaleManifest = previousManifestData?.defaultLocaleManifest
            return getBase(packageIdentifier, packageVersion, publisher, packageName, license, shortDescription, previousDefaultLocaleManifest).copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                packageLocale = defaultLocale
                    ?: previousManifestData?.versionManifest?.defaultLocale
                    ?: Locale.DEFAULT_LOCALE,
                publisher = publisher,
                publisherUrl = (publisherUrl
                    ?: previousDefaultLocaleManifest?.publisherUrl
                    ?: gitHubDetection?.publisherUrl)
                    ?.ifBlank { null },
                publisherSupportUrl = (previousDefaultLocaleManifest?.publisherSupportUrl
                    ?: gitHubDetection?.publisherSupportUrl
                    ?: pageScraper?.supportUrl?.await())
                    ?.ifBlank { null },
                privacyUrl = (previousDefaultLocaleManifest?.privacyUrl
                    ?: gitHubDetection?.privacyUrl
                    ?: pageScraper?.privacyUrl?.await())
                    ?.ifBlank { null },
                author = (author ?: previousDefaultLocaleManifest?.author)?.ifBlank { null },
                packageName = packageName,
                packageUrl = (packageUrl
                    ?: previousDefaultLocaleManifest?.packageUrl
                    ?: gitHubDetection?.packageUrl)
                    ?.ifBlank { null },
                license = license,
                licenseUrl = (licenseUrl
                    ?: previousDefaultLocaleManifest?.licenseUrl
                    ?: gitHubDetection?.licenseUrl)
                    ?.ifBlank { null },
                copyright = (copyright ?: previousDefaultLocaleManifest?.copyright)?.ifBlank { null },
                copyrightUrl = (copyrightUrl ?: previousDefaultLocaleManifest?.copyrightUrl)?.ifBlank { null },
                shortDescription = shortDescription,
                description = (description ?: previousDefaultLocaleManifest?.description)?.formatDescription(),
                moniker = (moniker ?: previousDefaultLocaleManifest?.moniker)?.ifBlank { null },
                tags = (tags ?: previousDefaultLocaleManifest?.tags)?.take(Tags.validationRules.maxItems)?.ifEmpty { null },
                releaseNotesUrl = (releaseNotesUrl
                    ?: gitHubDetection?.releaseNotesUrl
                    ?: parameterLocaleMetadata?.releaseNotesUrl)
                    ?.ifBlank { null },
                releaseNotes = (gitHubDetection?.releaseNotes ?: parameterLocaleMetadata?.releaseNotes)
                    ?.cutToCharLimitWithLines(ReleaseNotesFormatter.MAX_CHARACTERS)
                    ?.trim(),
                documentations = if (previousDefaultLocaleManifest?.documentations == null) {
                    listOfNotNull(
                        pageScraper?.faqUrl?.await()?.let {
                            Documentation(documentLabel = "FAQ", documentUrl = it)
                        }
                    )
                } else {
                    previousDefaultLocaleManifest.documentations
                }.ifEmpty { null },
                manifestType = SchemaType.DEFAULT_LOCALE,
                manifestVersion = manifestOverride ?: Schemas.MANIFEST_VERSION
            )
        }

        private inline fun Url.ifBlank(defaultValue: () -> Url?): Url? = if (this == Url("")) defaultValue() else this

        private fun String?.formatDescription() = this?.replace(Regex("([A-Z][a-z].*?[.!?]) ?(?=\$|[A-Z])"), "$1\n")
            ?.lines()
            ?.joinToString("\n") { it.trim() }
            ?.trim()
            ?.ifBlank { null }
    }
}
