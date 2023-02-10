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
import network.HttpUtils.fileNameWithoutExtension
import network.HttpUtils.getFileName
import org.kohsuke.github.GHAsset
import org.kohsuke.github.GHRelease
import org.kohsuke.github.PagedIterable
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import java.time.LocalDate
import java.time.ZoneId

class GitHubDetection(url: Url) : KoinComponent {
    private val pathSegments = url.pathSegments.filterNot { it.isBlank() }
    private val repository = CoroutineScope(Dispatchers.IO).async {
        get<GitHubImpl>().github.await().getRepository(/* name = */ "${pathSegments[0]}/${pathSegments[1]}")
    }
    private val release: Deferred<GHRelease?> = CoroutineScope(Dispatchers.IO).async {
        runCatching {
            repository.await().listReleases().find {
                it.tagName.contains(other = pathSegments.dropLast(1).last(), ignoreCase = true)
            }
        }.getOrNull()
    }
    private val assets = CoroutineScope(Dispatchers.IO).async { release.await()?.listAssets() }
    private val asset = CoroutineScope(Dispatchers.IO).async {
        assets.await()?.firstOrNull { it.browserDownloadUrl.decodeURLPart() == url.toString().decodeURLPart() }
    }

    var publisherUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        runCatching { repository.await().owner.blog }.getOrNull()?.let { Url(it) }
    }
    var shortDescription: Deferred<String?> = CoroutineScope(Dispatchers.IO).async { repository.await().description }
    var publisherSupportUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        if (repository.await().hasIssues()) Url("https://github.com/${repository.await().fullName}/issues") else null
    }
    var license: Deferred<String?> = CoroutineScope(Dispatchers.IO).async {
        runCatching {
            repository.await()
                .license
                ?.key
                ?.uppercase()
                ?.takeUnless { it.equals(other = "other", ignoreCase = true) }
        }.getOrNull()
    }
    var licenseUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        repository.await().licenseContent?.htmlUrl?.let { Url(it) }
    }
    var packageUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async { Url(repository.await().htmlUrl.toURI()) }
    var releaseDate: Deferred<LocalDate?> = CoroutineScope(Dispatchers.IO).async {
        runCatching {
            asset.await()?.let { LocalDate.ofInstant(it.createdAt.toInstant(), ZoneId.systemDefault()) }
        }.getOrNull()
    }
    var releaseNotesUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        release.await()?.htmlUrl?.let { Url(it.toURI()) }
    }
    var releaseNotes: Deferred<String?> = CoroutineScope(Dispatchers.IO).async {
        release.await()?.let { GitHubExtensions.getFormattedReleaseNotes(it) }
    }
    var privacyUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        repository.await()
            .getDirectoryContent("")
            .find { it.name.lowercase().contains(other = "privacy", ignoreCase = true) }
            ?.htmlUrl
            ?.let { Url(it) }
    }
    var topics: Deferred<List<String>?> = CoroutineScope(Dispatchers.IO).async { repository.await().listTopics() }
    var sha256: Deferred<String?> = CoroutineScope(Dispatchers.IO).async { findSha256(url, assets.await()) }

    private val client = get<Http>().client

    init {
        require(url.host.equals(other = gitHubWebsite, ignoreCase = true)) { "Url must be a GitHub Url" }
    }

    private suspend fun findSha256(url: Url, assets: PagedIterable<GHAsset>?): String? {
        return assets
            ?.find { it.isSha256(url) || it.name.equals(other = "Sha256Sums", ignoreCase = true) }
            ?.browserDownloadUrl
            ?.let { client.get(it).bodyAsText() }
            ?.let { Regex(pattern = "(.*) ${getFileName(url)}").find(it) }
            ?.groupValues
            ?.getOrNull(1)
            ?.trim()
    }

    private fun GHAsset.isSha256(url: Url): Boolean {
        return fileNameWithoutExtension(url)?.let {
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
