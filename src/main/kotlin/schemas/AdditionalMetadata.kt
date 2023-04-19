package schemas

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import java.time.LocalDate

@Serializable
data class AdditionalMetadata(
    @SerialName("Locales") val locales: List<Locale>? = null,
    @SerialName("ProductCode") val productCode: String? = null,
    @SerialName("ReleaseDate") @Contextual val releaseDate: LocalDate? = null,
    @SerialName("AppsAndFeaturesEntries")
    val appsAndFeaturesEntries: List<InstallerManifest.AppsAndFeaturesEntry>? = null
) {
    @Serializable
    data class Locale(
        @SerialName("Name") val name: String,
        @SerialName("Documentations") val documentations: List<LocaleManifest.Documentation>? = null,
        @SerialName("ReleaseNotes") val releaseNotes: String? = null,
        @SerialName("ReleaseNotesUrl") @Contextual val releaseNotesUrl: Url? = null
    )
}
