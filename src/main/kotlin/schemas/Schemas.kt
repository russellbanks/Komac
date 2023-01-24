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

    fun manifestType(installerSchema: InstallerSchema): String {
        return installerSchema.properties.manifestType.const
    }

    fun buildManifestString(schema: Schema, rawString: String): String {
        return buildString {
            appendLine(Comments.createdBy)
            appendLine(Comments.languageServer(schema))
            appendLine()
            appendLine(rawString)
        }
    }

    object Comments : KoinComponent {
        const val createdBy = "# Created with ${BuildConfig.appName} ${BuildConfig.appVersion}"
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
