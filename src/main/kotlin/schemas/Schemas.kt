package schemas

import com.russellbanks.Komac.BuildConfig
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.data.InstallerSchema

object Schemas {
    const val manifestVersion = "1.4.0"
    const val installerSchema = "https://aka.ms/winget-manifest.installer.$manifestVersion.schema.json"
    const val defaultLocaleSchema = "https://aka.ms/winget-manifest.defaultLocale.$manifestVersion.schema.json"
    const val localeSchema = "https://aka.ms/winget-manifest.locale.$manifestVersion.schema.json"
    const val versionSchema = "https://aka.ms/winget-manifest.version.$manifestVersion.schema.json"
    const val installerType = "installer"
    private const val customToolEnv = "KMC_CRTD_WITH"

    fun manifestType(installerSchema: InstallerSchema): String {
        return installerSchema.properties.manifestType.const
    }

    fun buildManifestString(schema: Schema, rawString: String): String {
        return buildString {
            append("# Created with ")
            System.getenv(customToolEnv)?.let { append("$it using ") }
            appendLine("${BuildConfig.appName} ${BuildConfig.appVersion}")
            appendLine(Comments.languageServer(schema))
            appendLine()
            appendLine(rawString)
        }
    }

    object Comments : KoinComponent {
        fun languageServer(schema: Schema): String {
            val schemaUrl = when (schema) {
                Schema.Installer -> installerSchema
                Schema.DefaultLocale -> defaultLocaleSchema
                Schema.Locale -> localeSchema
                Schema.Version -> versionSchema
            }.let { schemaUrl ->
                get<SchemasImpl>().manifestOverride?.let { schemaUrl.replace(manifestVersion, it) } ?: schemaUrl
            }
            return "# yaml-language-server: \$schema=$schemaUrl"
        }
    }
}
