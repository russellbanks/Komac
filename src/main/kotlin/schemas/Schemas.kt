package schemas

import Environment
import com.russellbanks.Komac.BuildConfig
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.Manifest
import schemas.manifest.VersionManifest

object Schemas {
    const val MANIFEST_VERSION = "1.4.0"
    private val installerSchema get() = "https://aka.ms/winget-manifest.installer.$MANIFEST_VERSION.schema.json"
    private val defaultLocaleSchema get() = "https://aka.ms/winget-manifest.defaultLocale.$MANIFEST_VERSION.schema.json"
    private val localeSchema get() = "https://aka.ms/winget-manifest.locale.$MANIFEST_VERSION.schema.json"
    private val versionSchema get() = "https://aka.ms/winget-manifest.version.$MANIFEST_VERSION.schema.json"
    const val INSTALLER_MANIFEST_TYPE = "installer"
    const val DEFAULT_LOCALE_MANIFEST_TYPE = "defaultLocale"
    const val VERSION_MANIFEST_TYPE = "version"
    const val MANIFEST_VERSION_REGEX = "^(0|[1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])(\\.(0|[1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])){2}$"

    fun buildManifestString(manifest: Manifest, rawString: String): String = buildString {
        append("# Created with ")
        Environment.customToolName?.let { append("$it using ") }
        appendLine("${BuildConfig.appName} v${BuildConfig.appVersion}")
        appendLine(languageServer(manifest))
        appendLine()
        appendLine(rawString)
    }

    private fun languageServer(manifest: Manifest): String {
        val schemaUrl = when (manifest) {
            is InstallerManifest -> installerSchema
            is DefaultLocaleManifest -> defaultLocaleSchema
            is LocaleManifest -> localeSchema
            is VersionManifest -> versionSchema
        }
        return "# yaml-language-server: \$schema=$schemaUrl"
    }
}
