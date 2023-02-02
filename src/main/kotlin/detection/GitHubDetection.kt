package detection

import data.GitHubImpl
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Url
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import network.Http
import network.HttpUtils.decodeHex
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
        assets.await()?.firstOrNull { Url(it.browserDownloadUrl).decodeHex() == url.decodeHex() }
    }

    var publisherUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        if (repository.await().hasIssues()) Url("https://github.com/${repository.await().fullName}/issues") else null
    }
    var shortDescription: Deferred<String?> = CoroutineScope(Dispatchers.IO).async { repository.await().description }
    var publisherSupportUrl: Deferred<Url?> = CoroutineScope(Dispatchers.IO).async {
        runCatching { repository.await().owner.blog }.getOrNull()?.let { Url(it) }
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
        release.await()?.let { getFormattedReleaseNotes(it) }
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

    /**
     * Extracts formatted release notes from a given release.
     *
     * 1. The function first splits the release notes body into lines and cleans each line by removing dropdowns,
     * changing all bullet points to be dashes, removing code formatted with backticks, and converting Markdown links
     * to plaintext.
     * 2. It then uses a buildString block to loop through each line of the release notes.
     * 3. If the line starts with "#" and there is another bullet point within two lines of it, it is added.
     * 4. If the line starts with "- " it is added, with each sentence being on a new line and indented.
     * 5. Finally, either the string is returned, or null if it is blank.
     *
     * @param release the [GHRelease] object containing the release notes to be formatted
     * @return A formatted string of the release notes or null if the release notes are blank
     */
    private fun getFormattedReleaseNotes(release: GHRelease): String? {
        val lines = release.body
            ?.replace(Regex("<details>.*?</details>", setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)), "")
            ?.lines()
            ?.map { line ->
                line.trim()
                    .let { if (it.startsWith("* ")) it.replaceFirst("* ", "- ") else it }
                    .replace(Regex("""\*+([^*]+)\*+"""), "$1")
                    .replace("`", "")
                    .replace(Regex("\\[([^]]+)]\\([^)]+\\)"), "$1")
            }
        return buildString {
            lines?.forEachIndexed { index, line ->
                when {
                    line.startsWith("#") -> {
                        if (lines[index + 1].startsWith("- ") || lines[index + 2].startsWith("- ")) {
                            line.dropWhile { it == '#' }.trim().takeUnless { it.isBlank() }?.let { appendLine(it) }
                        }
                    }
                    line.startsWith("- ") -> {
                        appendLine(
                            "- ${line.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n  ").drop(2).trim()}"
                        )
                    }
                }
            }
        }.ifBlank { null }
    }

    companion object {
        const val gitHubWebsite = "github.com"
    }
}
