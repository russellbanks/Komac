package detection

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import kotlinx.coroutines.runBlocking
import network.WebPageScraper

class PageScraperTest : FunSpec({
    val testHtml = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
        </head>
        <body>
            <a href="https://www.example.com/support">Support</a>
            <a href="https://www.example.com/help">Help</a>
            <a href="https://www.example.com/faq">FAQ</a>
            <a href="https://www.example.com/privacy">Privacy</a>
        </body>
        </html>
    """.trimIndent()

    val mockClient = HttpClient(MockEngine) {
        engine {
            addHandler { _ ->
                respond(testHtml, HttpStatusCode.OK)
            }
        }
    }

    val scraper = WebPageScraper(Url("https://www.example.com"), mockClient)

    test("support url should be found") {
        runBlocking {
            val supportUrl = scraper.supportUrl.await()
            supportUrl.shouldNotBeNull()
            supportUrl.toString() shouldBe "https://www.example.com/support"
        }
    }

    test("faq url should be found") {
        runBlocking {
            val faqUrl = scraper.faqUrl.await()
            faqUrl.shouldNotBeNull()
            faqUrl.toString() shouldBe "https://www.example.com/faq"
        }
    }

    test("privacy url should be found") {
        runBlocking {
            val privacyUrl = scraper.privacyUrl.await()
            privacyUrl.shouldNotBeNull()
            privacyUrl.toString() shouldBe "https://www.example.com/privacy"
        }
    }
})
