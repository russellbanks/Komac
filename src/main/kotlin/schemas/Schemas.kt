package schemas

object Schemas {
    const val manifestVersion = "1.4.0"
    const val versionSchema = "https://aka.ms/winget-manifest.version.$manifestVersion.schema.json"
    const val defaultLocaleSchema = "https://aka.ms/winget-manifest.defaultLocale.$manifestVersion.schema.json"
    const val installerSchema = "https://aka.ms/winget-manifest.installer.$manifestVersion.schema.json"
    const val localeSchema = "https://aka.ms/winget-manifest.locale.$manifestVersion.schema.json"

    object Comments {
        const val createdBy = "# Created using Komac"
        const val installerLanguageServer = "# yaml-language-server: \$schema=$installerSchema"
    }
}
