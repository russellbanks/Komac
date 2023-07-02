package github

import github.ReleaseNotesFormatter.formattedReleaseNotes
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Url
import io.ktor.http.decodeURLPart
import java.net.URL
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toKotlinInstant
import kotlinx.datetime.toLocalDateTime
import network.Http
import org.kohsuke.github.GHAsset
import org.kohsuke.github.GHRelease
import org.kohsuke.github.PagedIterable
import utils.extension
import utils.getFileNameWithoutExtension

class GitHubDetection(url: Url) {
    // Properties
    private val pathSegments = url.pathSegments.filterNot(String::isBlank)
    private val repository = GitHubImpl.github.getRepository("${pathSegments[0]}/${pathSegments[1]}")
    private val release: GHRelease? = findRelease()
    private val assets = release?.listAssets()
    private val asset = findAsset(url)

    var sha256: String? = null
        private set

    var publisherUrl: Url? = findPublisherUrl()
    var shortDescription: String? = repository.description
    var publisherSupportUrl: Url? = findPublisherSupportUrl()
    var license: String? = findLicense()
    var licenseUrl: Url? = findLicenseUrl()
    var packageUrl: Url? = findPackageUrl()
    var releaseDate: LocalDate? = findReleaseDate()
    var releaseNotesUrl: Url? = findReleaseNotesUrl()
    var releaseNotes: String? = release?.formattedReleaseNotes
    var privacyUrl: Url? = findPrivacyUrl()
    var topics: List<String>? = findTopics()

    init {
        require(url.host.equals(gitHubWebsite, ignoreCase = true)) { "Url must be a GitHub Url" }
        CoroutineScope(Dispatchers.IO).launch { sha256 = findSha256(url, assets) }
    }

    // Functions
    private fun findRelease(): GHRelease? = runCatching {
        repository.listReleases().find {
            it.tagName.contains(pathSegments.dropLast(1).last(), ignoreCase = true)
        }
    }.getOrNull()

    private fun findAsset(url: Url): GHAsset? = assets?.firstOrNull {
        it.browserDownloadUrl.decodeURLPart() == url.toString().decodeURLPart()
    }

    private fun findPublisherUrl(): Url? = runCatching {
        if (repository.homepage != null) {
            repository.homepage?.let(::Url)
        } else {
            repository.owner.blog?.let(::Url)
        }
    }.getOrNull()

    private fun findPublisherSupportUrl(): Url? = if (repository.hasIssues()) {
        Url("https://github.com/${repository.fullName}/issues")
    } else {
        null
    }

    private fun findLicense(): String? = runCatching {
        repository.license
            ?.key
            ?.uppercase()
            ?.takeUnless { it.equals("other", ignoreCase = true) }
    }.getOrNull()

    private fun findLicenseUrl(): Url? = repository.licenseContent?.htmlUrl?.let(::Url)

    private fun findPackageUrl(): Url? = repository.htmlUrl.toURI()?.let(::Url)

    private fun findReleaseDate(): LocalDate? = runCatching {
        asset?.createdAt?.toInstant()?.toKotlinInstant()?.toLocalDateTime(TimeZone.UTC)?.date
    }.getOrNull()

    private fun findReleaseNotesUrl(): Url? = release?.htmlUrl?.let(URL::toURI)?.let(::Url)

    private fun findPrivacyUrl(): Url? = runCatching {
        repository
            .getDirectoryContent("")
            .find { it.name.lowercase().contains("privacy", ignoreCase = true) }
            ?.htmlUrl
            ?.let(::Url)
    }.getOrNull()

    private fun findTopics(): List<String>? = runCatching { repository.listTopics() }.getOrNull()

    private suspend fun findSha256(url: Url, assets: PagedIterable<GHAsset>?): String? {
        return assets
            ?.find { it.isSha256(url) || it.name.equals("Sha256Sums", ignoreCase = true) }
            ?.browserDownloadUrl
            ?.let { Http.client.get(it).bodyAsText() }
            ?.let("(.*) ${url.extension}".toRegex()::find)
            ?.groupValues
            ?.getOrNull(1)
            ?.trim()
    }

    private fun GHAsset.isSha256(url: Url): Boolean {
        return url.getFileNameWithoutExtension()?.let {
            name.contains(it, ignoreCase = true)
        } == true && endsWithSha256()
    }

    private fun GHAsset.endsWithSha256(): Boolean {
        return name.endsWith(".sha256sum", ignoreCase = true) ||
            name.endsWith(".sha256", ignoreCase = true)
    }

    companion object {
        const val gitHubWebsite = "github.com"
    }
}
