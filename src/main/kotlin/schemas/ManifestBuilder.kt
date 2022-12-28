package schemas

import data.SharedManifestData
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

object ManifestBuilder : KoinComponent {
    val sharedManifestData: SharedManifestData by inject()
    val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    val defaultLocaleManifestName = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    val versionManifestName = "${sharedManifestData.packageIdentifier}.version.yaml"

    private val baseGitHubPath = buildString {
        append("manifests/")
        append("${sharedManifestData.packageIdentifier.first().lowercase()}/")
        append("${sharedManifestData.packageIdentifier.replace(".", "/")}/")
        append(sharedManifestData.packageVersion)
    }

    val installerManifestGitHubPath = "$baseGitHubPath/$installerManifestName"

    val defaultLocaleManifestGitHubPath = "$baseGitHubPath/$defaultLocaleManifestName"

    val versionManifestGitHubPath = "$baseGitHubPath/$versionManifestName"

    fun buildManifestString(schemaUrl: String, block: StringBuilder.() -> Unit): String {
        return buildString {
            appendLine(Schemas.Comments.createdBy)
            appendLine(Schemas.Comments.languageServer(schemaUrl))
            appendLine()
            block()
        }
    }
}
