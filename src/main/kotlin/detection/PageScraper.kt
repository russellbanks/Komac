package detection

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
import network.Http
import org.koin.core.component.KoinComponent
import org.koin.core.component.get

/**
 * Page Scraper class that finds Locale Manifest Urls from the root page of the provided Url.
 *
 * This will always search the root of the website, regardless of what path segments the provided Url has.
 *
 * @param url of the website
 */
class PageScraper(url: Url) : KoinComponent {
    private val urlRoot = URLBuilder(url).apply { pathSegments = emptyList() }.build()
    private val client = get<Http>().client
    private val scope = CoroutineScope(Dispatchers.IO)
    private var linksMap: Deferred<HashMap<String, String>> = scope.async {
        htmlDocument(client.get(urlRoot).bodyAsText()) {
            a {
                findAll {
                    eachLink as HashMap<String, String>
                }
            }
        }
    }
    var supportUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(support, help) }
    var faqUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(faq) }
    var privacyUrl: Deferred<Url?> = scope.async { getUrlForSearchValue(privacy) }

    /**
     * This abstracts [findFirstLink] and cleans up the returned String to ensure that it is always in the format of
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
            else -> URLBuilder(urlRoot).apply { pathSegments = result.split("/") }.build()
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
        var result: String? = null
        outer@ for (searchValue in searchValues) {
            for (value in linksMap.values) {
                if (value.contains(searchValue, ignoreCase = true)) {
                    result = value
                    break@outer
                }
            }
            for (key in linksMap.keys) {
                if (key.contains(searchValue, ignoreCase = true)) {
                    result = linksMap[key]
                    break@outer
                }
            }
        }
        return result
    }

    companion object {
        // Support
        private const val support = "Support"
        private const val help = "Help"

        // Privacy
        private const val privacy = "Privacy"

        // FAQ
        private const val faq = "FAQ"
    }
}
