package schemas

import io.ktor.http.Url
import kotlinx.datetime.LocalDate
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import schemas.manifest.InstallerManifest

@Serializable
data class AdditionalMetadata(
    val locales: List<Locale>? = null,
    val productCode: String? = null,
    val releaseDate: LocalDate? = null,
    val appsAndFeaturesEntries: InstallerManifest.AppsAndFeaturesEntry? = null
) {
    @Serializable
    data class Locale(
        val name: String,
        val releaseNotes: String? = null,
        @Contextual val releaseNotesUrl: Url? = null
    )
}
