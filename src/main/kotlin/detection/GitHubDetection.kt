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
            val release: GHRelease = repository.getReleaseByTagName(tag)
            val asset = release.listAssets().first { it.browserDownloadUrl == url.toString() }
            releaseDate = async { LocalDate.ofInstant(asset.createdAt.toInstant(), ZoneId.systemDefault()) }
            license = async { repository.license?.key?.uppercase() }
            packageUrl = async { Url(repository.htmlUrl.toURI()) }
            licenseUrl = async { repository.licenseContent?.htmlUrl?.let { Url(it) } }
            privacyUrl = async {
                repository
                    .getDirectoryContent("")
                    .find { it.name.lowercase().contains("privacy") }
                    ?.htmlUrl
                    ?.let { Url(it) }
            }
            releaseNotesUrl = async { Url(release.htmlUrl.toURI()) }
            releaseNotes = async { getFormattedReleaseNotes(release) }
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
     * 1. The function first splits the release notes body into lines and initializes a variable called "title" to an
     * empty string and "titleAdded" to false.
     * 2. It then uses a buildString block to loop through each line of the release notes.
     * 3. If the line starts with "#", the title variable is updated to the text after the "#" and titleAdded is set to
     * false.
     * 4. If the line starts with "- " or "* ", it is added to the string with a "- " prefix, and if the title has not
     * been added yet, it is also added to the string.
     * 5. If a title has been added and the next two lines do not start with "#", the title is removed from the string
     * and the titleAdded variable is set to false.
     * 6. Finally, the string has all instances of "text" removed and is trimmed, returning null if the string is empty.
     *
     * @param release the [GHRelease] object containing the release notes to be formatted
     * @return A formatted string of the release notes or null if the release notes are blank
     */
    private fun getFormattedReleaseNotes(release: GHRelease): String? {
        val lines = release.body.lines()
        var title = ""
        var titleAdded = false
        return buildString {
            lines.forEachIndexed { index, line ->
                val cleanedLine = line.trim()
                if (cleanedLine.startsWith("#")) {
                    title = cleanedLine.dropWhile { it == '#' }.trim().ifEmpty { "" }
                    titleAdded = false
                } else if (cleanedLine.startsWith("- ") || cleanedLine.startsWith("* ")) {
                    if (!titleAdded && title.isNotEmpty()) {
                        appendLine(title)
                        titleAdded = true
                    }
                    appendLine("- ${cleanedLine.substring(2).trim()}")
                } else if (
                    titleAdded && (index < lines.size - 2 &&
                        !lines[index + 1].startsWith("#") &&
                        !lines[index + 2].startsWith("#"))
                ) {
                    delete(length - title.length - 1, length)
                    title = ""
                    titleAdded = false
                }
            }
        }.replace(Regex("\\[([^]]+)]\\([^)]+\\)"), "$1").trim().ifBlank { null }
    }

    companion object {
        const val gitHubWebsite = "github.com"
    }
}
