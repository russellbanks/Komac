package data.locale

enum class LocaleUrl {
    CopyrightUrl,
    LicenseUrl,
    PackageUrl,
    PublisherUrl,
    PublisherSupportUrl,
    PublisherPrivacyUrl,
    ReleaseNotesUrl;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
