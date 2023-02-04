package schemas

import io.ktor.http.Url
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import java.time.LocalDate

@Serializable
data class AdditionalMetadata(
    val locales: List<Locale>? = null,
    val productCode: String? = null,
    @Contextual val releaseDate: LocalDate? = null,
    val appsAndFeaturesEntries: List<InstallerManifest.Installer.AppsAndFeaturesEntry>? = null
) {
    @Serializable
    data class Locale(
        val name: String,
        val documentations: List<LocaleManifest.Documentation>? = null,
        val releaseNotes: String? = null,
        @Contextual val releaseNotesUrl: Url? = null
    )
}
