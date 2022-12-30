package schemas

import data.InstallerManifestData
import data.SharedManifestData
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

object ManifestBuilder : KoinComponent {
    val sharedManifestData: SharedManifestData by inject()
    val installerManifestData: InstallerManifestData by inject()
    private val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    private val defaultLocaleManifestName
        get() = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    private val versionManifestName = "${sharedManifestData.packageIdentifier}.version.yaml"

    private val baseGitHubPath = buildString {
        append("manifests/")
        append("${sharedManifestData.packageIdentifier.first().lowercase()}/")
        append("${sharedManifestData.packageIdentifier.replace(".", "/")}/")
        append(sharedManifestData.packageVersion)
    }

    val installerManifestGitHubPath = "$baseGitHubPath/$installerManifestName"

    val defaultLocaleManifestGitHubPath = "$baseGitHubPath/$defaultLocaleManifestName"

    val versionManifestGitHubPath = "$baseGitHubPath/$versionManifestName"

    fun getLocaleManifestGitHubPath(locale: String): String {
        return "$baseGitHubPath/${sharedManifestData.packageIdentifier}.locale.$locale.yaml"
    }
}
