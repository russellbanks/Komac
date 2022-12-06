package schemas

import com.russellbanks.Komac.BuildConfig

object Schemas {
    const val manifestVersion = "1.4.0"
    const val versionSchema = "https://aka.ms/winget-manifest.version.$manifestVersion.schema.json"
    const val defaultLocaleSchema = "https://aka.ms/winget-manifest.defaultLocale.$manifestVersion.schema.json"
    const val installerSchema = "https://aka.ms/winget-manifest.installer.$manifestVersion.schema.json"
    const val localeSchema = "https://aka.ms/winget-manifest.locale.$manifestVersion.schema.json"

    object Comments {
        const val createdBy = "# Created using ${BuildConfig.appName} ${BuildConfig.appVersion}"
        const val installerLanguageServer = "# yaml-language-server: \$schema=$installerSchema"
    }

    object InstallerType {
        const val exe = "exe"
    }
}
