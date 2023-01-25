package detection

import data.GitHubImpl
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import io.ktor.http.appendPathSegments
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import ktor.Ktor.decodeHex
import org.kohsuke.github.GHRelease
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import java.time.LocalDate
import java.time.ZoneId

class GitHubDetection(url: Url) : KoinComponent {
    var publisherUrl: Deferred<Url?>? = null
    var shortDescription: Deferred<String?>? = null
    var publisherSupportUrl: Deferred<Url?>? = null
    var license: Deferred<String?>? = null
    var licenseUrl: Deferred<Url?>? = null
    var packageUrl: Deferred<Url?>? = null
    var releaseDate: Deferred<LocalDate?>? = null
    var releaseNotesUrl: Deferred<Url?>? = null
    var releaseNotes: Deferred<String?>? = null
    var privacyUrl: Deferred<Url?>? = null
    var topics: Deferred<List<String>?>? = null

    private val githubImpl: GitHubImpl by inject()

    init {
        require(url.host.equals(other = gitHubWebsite, ignoreCase = true)) { "Url must be a GitHub Url" }
        CoroutineScope(Dispatchers.IO).launch {
            val tag = url.pathSegments.dropLast(1).last()
            val repository = githubImpl.github.await().getRepository("${url.pathSegments[1]}/${url.pathSegments[2]}")
            val release: GHRelease? = runCatching { repository.getReleaseByTagName(tag) }.getOrNull()
            val asset = release?.listAssets()?.firstOrNull { Url(it.browserDownloadUrl).decodeHex() == url.decodeHex() }
            releaseDate = async { asset?.let { LocalDate.ofInstant(it.createdAt.toInstant(), ZoneId.systemDefault()) } }
            license = async {
                runCatching {
                    repository.license?.key?.uppercase()?.takeUnless { it.equals(other = "other", ignoreCase = true) }
                }.getOrNull()
            }
            packageUrl = async { Url(repository.htmlUrl.toURI()) }
            licenseUrl = async { repository.licenseContent?.htmlUrl?.let { Url(it) } }
            privacyUrl = async {
                repository
                    .getDirectoryContent("")
                    .find { it.name.lowercase().contains(other = "privacy", ignoreCase = true) }
                    ?.htmlUrl
                    ?.let { Url(it) }
            }
            releaseNotesUrl = async { release?.htmlUrl?.let { Url(it.toURI()) } }
            releaseNotes = async { release?.let { getFormattedReleaseNotes(it) } }
            topics = async { repository.listTopics() }
            publisherUrl = async { runCatching { repository.owner.blog }.getOrNull()?.let { Url(it) } }
            shortDescription = async { repository.description }
            publisherSupportUrl = async {
                val supportUrl = URLBuilder(url).appendPathSegments("support").build()
                data.shared.Url.isUrlValid(
                    url = supportUrl,
                    schema = get<SchemasImpl>().defaultLocaleSchema,
                    canBeBlank = false
                ).let { error ->
                    when {
                        error == null -> supportUrl
                        repository.hasIssues() -> Url("https://github.com/${repository.fullName}/issues")
                        else -> null
                    }
                }
            }
        }
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
            .replace(Regex("<details>.*?</details>", setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)), "")
            .lines()
            .map { line ->
                line.trim()
                    .let { if (it.startsWith("* ")) it.replaceFirst("* ", "- ") else it }
                    .replace("`", "")
                    .replace(Regex("\\[([^]]+)]\\([^)]+\\)"), "$1")
            }
        return buildString {
            lines.forEachIndexed { index, line ->
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
