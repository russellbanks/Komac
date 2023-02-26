package detection.github

import data.GitHubImpl
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Url
import io.ktor.http.decodeURLPart
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import network.Http
import network.getExtension
import network.getFileNameWithoutExtension
import org.kohsuke.github.GHAsset
import org.kohsuke.github.GHRelease
import org.kohsuke.github.PagedIterable
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import java.time.LocalDate
import java.time.ZoneOffset

class GitHubDetection(url: Url) : KoinComponent {
    private val pathSegments = url.pathSegments.filterNot { it.isBlank() }
    private val repository = get<GitHubImpl>().github.getRepository("${pathSegments[0]}/${pathSegments[1]}")
    private val release: GHRelease? = runCatching {
        repository.listReleases().find {
            it.tagName.contains(other = pathSegments.dropLast(1).last(), ignoreCase = true)
        }
    }.getOrNull()
    private val assets = release?.listAssets()
    private val asset = assets?.firstOrNull { it.browserDownloadUrl.decodeURLPart() == url.toString().decodeURLPart() }
    var publisherUrl: Url? = runCatching { repository.owner.blog }.getOrNull()?.let { Url(it) }
    var shortDescription: String? = repository.description
    var publisherSupportUrl: Url? = if (repository.hasIssues()) {
        Url("https://github.com/${repository.fullName}/issues")
    } else {
        null
    }
    var license: String? = runCatching {
        repository
            .license
            ?.key
            ?.uppercase()
            ?.takeUnless { it.equals(other = "other", ignoreCase = true) }
    }.getOrNull()
    var licenseUrl: Url? = repository.licenseContent?.htmlUrl?.let { Url(it) }
    var packageUrl: Url? = Url(repository.htmlUrl.toURI())
    var releaseDate: LocalDate? = runCatching {
        asset?.createdAt?.toInstant()?.atOffset(ZoneOffset.UTC)?.toLocalDate()
    }.getOrNull()
    var releaseNotesUrl: Url? = release?.htmlUrl?.let { Url(it.toURI()) }
    var releaseNotes: String? = release?.let { GitHubExtensions.getFormattedReleaseNotes(it) }
    var privacyUrl: Url? = runCatching {
        repository
            .getDirectoryContent("")
            .find { it.name.lowercase().contains(other = "privacy", ignoreCase = true) }
            ?.htmlUrl
            ?.let { Url(it) }
    }.getOrNull()
    var topics: List<String>? = runCatching { repository.listTopics() }.getOrNull()
    var sha256: Deferred<String?> = CoroutineScope(Dispatchers.IO).async { findSha256(url, assets) }

    private val client = get<Http>().client

    init {
        require(url.host.equals(other = gitHubWebsite, ignoreCase = true)) { "Url must be a GitHub Url" }
    }

    private suspend fun findSha256(url: Url, assets: PagedIterable<GHAsset>?): String? {
        return assets
            ?.find { it.isSha256(url) || it.name.equals(other = "Sha256Sums", ignoreCase = true) }
            ?.browserDownloadUrl
            ?.let { client.get(it).bodyAsText() }
            ?.let { Regex(pattern = "(.*) ${url.getExtension()}").find(it) }
            ?.groupValues
            ?.getOrNull(1)
            ?.trim()
    }

    private fun GHAsset.isSha256(url: Url): Boolean {
        return url.getFileNameWithoutExtension()?.let {
            name.contains(other = it, ignoreCase = true)
        } == true && endsWithSha256()
    }

    private fun GHAsset.endsWithSha256(): Boolean {
        return name.endsWith(suffix = ".sha256sum", ignoreCase = true) ||
            name.endsWith(suffix = ".sha256", ignoreCase = true)
    }

    companion object {
        const val gitHubWebsite = "github.com"
    }
}
