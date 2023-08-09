package schemas.manifest

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.Schemas

/**
 * A representation of a multiple-file manifest representing app metadata in other locale in the OWC. v1.5.0
 */
@Suppress("unused")
@Serializable
data class LocaleManifest(
    val packageIdentifier: String,
    val packageVersion: String,
    val packageLocale: String,
    val publisher: String? = null,
    @Contextual val publisherUrl: Url? = null,
    @Contextual val publisherSupportUrl: Url? = null,
    @Contextual val privacyUrl: Url? = null,
    val author: String? = null,
    val packageName: String? = null,
    @Contextual val packageUrl: Url? = null,
    val license: String? = null,
    @Contextual val licenseUrl: Url? = null,
    val copyright: String? = null,
    @Contextual val copyrightUrl: Url? = null,
    val shortDescription: String? = null,
    val description: String? = null,
    val tags: List<String>? = null,
    val agreements: List<Agreement>? = null,
    val releaseNotes: String? = null,
    @Contextual val releaseNotesUrl: Url? = null,
    @Contextual val purchaseUrl: Url? = null,
    val installationNotes: String? = null,
    val documentations: List<Documentation>? = null,
    val icons: List<Icon>? = null,
    val manifestType: String,
    val manifestVersion: String
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
            Custom,
            @SerialName("16x16") Size16,
            @SerialName("20x20") Size20,
            @SerialName("24x24") Size24,
            @SerialName("30x30") Size30,
            @SerialName("32x32") Size32,
            @SerialName("36x36") Size36,
            @SerialName("40x40") Size40,
            @SerialName("48x48") Size48,
            @SerialName("60x60") Size60,
            @SerialName("64x64") Size64,
            @SerialName("72x72") Size72,
            @SerialName("80x80") Size80,
            @SerialName("96x96") Size96,
            @SerialName("256x256") Size256
        }

        enum class IconTheme {
            Default,
            Light,
            Dark,
            HighContrast
        }
    }

    override fun toString() = Schemas.buildManifestString(
        manifest = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )

    companion object {
        /**
         * Returns the name of the YAML file containing the localized manifest for the given package identifier and locale.
         *
         * @param identifier the package identifier to get the manifest name for
         * @param locale the locale to get the manifest name for
         * @return a string representing the name of the YAML file containing the localized manifest for the given
         * identifier and locale
         */
        fun getFileName(identifier: String, locale: String) = "$identifier.locale.$locale.yaml"
    }
}
