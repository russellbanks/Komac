package data.locale

enum class LocaleUrl {
    CopyrightUrl,
    LicenseUrl,
    PackageUrl,
    PublisherUrl,
    PublisherSupportUrl,
    PublisherPrivacyUrl;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
