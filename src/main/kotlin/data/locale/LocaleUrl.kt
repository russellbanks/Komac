package data.locale

enum class LocaleUrl {
    LicenseUrl,
    PackageUrl,
    PublisherUrl,
    PublisherSupportUrl,
    PublisherPrivacyUrl;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
