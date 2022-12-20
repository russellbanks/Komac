package data.locale

enum class PublisherUrl {
    PublisherUrl,
    PublisherSupportUrl,
    PublisherPrivacyUrl;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
