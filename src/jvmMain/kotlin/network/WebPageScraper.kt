package network

import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import it.skrape.core.htmlDocument
import it.skrape.selects.eachLink
import it.skrape.selects.html5.a
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async

/**
 * Page Scraper class that finds Locale Manifest Urls from the root page of the provided Url.
 *
 * This will always search the root of the website, regardless of what path segments the provided Url has.
 *
 * @param url of the website
 */
class WebPageScraper(url: Url, private val client: HttpClient = Http.client) {
    private val urlRoot = URLBuilder(url).apply {
        host = host.split('.').takeLast(2).joinToString(".")
        pathSegments = emptyList()
    }.build()
    private val scope = CoroutineScope(Dispatchers.IO)
    private val linksMap: Deferred<Map<String, String>?> = scope.async {
        runCatching { parseLinks(client.get(urlRoot).bodyAsText()).getOrDefault(HashMap()) }.getOrNull()
    }
    val supportUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(SUPPORT, HELP) }
    val faqUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(FAQ) }
    val privacyUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(PRIVACY) }

    /**
     * Parses an HTML document into a map of link text to href values.
     *
     * @param html the HTML document to parse
     * @return a map of link text to href values
     */
    private fun parseLinks(html: String): Result<Map<String, String>> = runCatching {
        htmlDocument(html) {
            a {
                findAll {
                    eachLink
                }
            }
        }
    }

    /**
     * This abstracts [findFirstLink] and ensures that the returned String is always in the format of
     * https://www.host.com.
     *
     * @param searchValues the values to sequentially search for
     * @return the link as a [Url] or null if no matching links are found
     */
    private suspend fun getUrlForSearchValue(vararg searchValues: String): Url? {
        val result = findFirstLink(*searchValues)?.removeSuffix("/")
        return when {
            result == null -> null
            result.startsWith("https://") -> Url(result)
            result.startsWith("http://") -> Url(result.replace("http://", "https://"))
            else -> URLBuilder(urlRoot).apply { pathSegments = result.split('/') }.build()
        }
    }

    /**
     * Hierarchically finds the first link that matches the search value.
     *
     * It works by checking the values in the [linksMap] for the search value.
     * If there are none that match, it checks the keys. If there is a match, it returns the associated value.
     * If there are no matches, the process is repeated for the next search value.
     *
     * @param searchValues the values to sequentially search for
     * @return the link as a [String] or null if no matching links are found
     */
    private suspend fun findFirstLink(vararg searchValues: String): String? {
        val linksMap = linksMap.await()
        return searchValues.asSequence()
            .mapNotNull { searchValue ->
                linksMap?.entries?.firstOrNull { (key, value) ->
                    key.contains(searchValue, ignoreCase = true) || value.contains(searchValue, ignoreCase = true)
                }?.value
            }
            .firstOrNull()
    }

    companion object {
        // Support
        private const val SUPPORT = "Support"
        private const val HELP = "Help"

        // Privacy
        private const val PRIVACY = "Privacy"

        // FAQ
        private const val FAQ = "FAQ"
    }
}
