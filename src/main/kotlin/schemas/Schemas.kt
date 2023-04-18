package schemas

import com.russellbanks.Komac.BuildConfig

object Schemas {
    var manifestVersion = "1.4.0"
    private val installerSchema get() = "https://aka.ms/winget-manifest.installer.$manifestVersion.schema.json"
    private val defaultLocaleSchema get() = "https://aka.ms/winget-manifest.defaultLocale.$manifestVersion.schema.json"
    private val localeSchema get() = "https://aka.ms/winget-manifest.locale.$manifestVersion.schema.json"
    private val versionSchema get() = "https://aka.ms/winget-manifest.version.$manifestVersion.schema.json"
    const val installerManifestType = "installer"
    const val defaultLocaleManifestType = "defaultLocale"
    const val versionManifestType = "version"
    const val customToolEnv = "KMC_CRTD_WITH"

    fun buildManifestString(schema: Schema, rawString: String): String {
        return buildString {
            append("# Created with ")
            System.getenv(customToolEnv)?.let { append("$it using ") }
            appendLine("${BuildConfig.appName} v${BuildConfig.appVersion}")
            appendLine(languageServer(schema))
            appendLine()
            appendLine(rawString)
        }
    }

    private fun languageServer(schema: Schema): String {
        val schemaUrl = when (schema) {
            Schema.Installer -> installerSchema
            Schema.DefaultLocale -> defaultLocaleSchema
            Schema.Locale -> localeSchema
            Schema.Version -> versionSchema
        }
        return "# yaml-language-server: \$schema=$schemaUrl"
    }
}
