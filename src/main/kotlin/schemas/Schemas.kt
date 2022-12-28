package schemas

import com.russellbanks.Komac.BuildConfig

object Schemas {
    private const val manifestVersion = "1.4.0"
    const val installerSchema = "https://aka.ms/winget-manifest.installer.$manifestVersion.schema.json"
    const val defaultLocaleSchema = "https://aka.ms/winget-manifest.defaultLocale.$manifestVersion.schema.json"
    const val localeSchema = "https://aka.ms/winget-manifest.locale.$manifestVersion.schema.json"
    const val versionSchema = "https://aka.ms/winget-manifest.version.$manifestVersion.schema.json"

    fun manifestType(installerSchema: InstallerSchema): String {
        return installerSchema.properties.manifestType.const
    }

    object Comments {
        const val createdBy = "# Created using ${BuildConfig.appName} ${BuildConfig.appVersion}"
        fun languageServer(schemaUrl: String) = "# yaml-language-server: \$schema=$schemaUrl"
    }

    object InstallerType {
        const val exe = "exe"
    }
}
