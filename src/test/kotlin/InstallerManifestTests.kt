
import io.kotest.assertions.ktor.client.shouldHaveStatus
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldNotBe
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpStatusCode
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import schemas.InstallerSchema
import schemas.LocaleSchema
import schemas.Schemas
import schemas.VersionSchema

class InstallerManifestTests : FunSpec({
    val client = HttpClient(Java) {
        install(UserAgent) {
            agent = "Microsoft-Delivery-Optimization/10.1"
        }
    }

    val json = Json { ignoreUnknownKeys = true }
    var installerSchema: InstallerSchema? = null
    var localeSchema: LocaleSchema? = null
    var versionSchema: VersionSchema? = null

    listOf(
        Schemas.installerSchema to installerSchema,
        Schemas.localeSchema to localeSchema,
        Schemas.versionSchema to versionSchema
    ).forEach {
        context("Get ${it.first}") {
            lateinit var response: HttpResponse

            test("Retrieve ${it.first}") {
                response = client.get(it.first)
                with (response) {
                    shouldNotBeNull()
                    shouldHaveStatus(HttpStatusCode.OK)
                }
            }

            test("Parse ${it.first}") {
                when (it.second) {
                    Schemas.installerSchema -> {
                        installerSchema = json.decodeFromString(response.body())
                        installerSchema shouldNotBe null
                    }
                    Schemas.localeSchema -> {
                        localeSchema = json.decodeFromString(response.body())
                        localeSchema shouldNotBe null
                    }
                    Schemas.versionSchema -> {
                        versionSchema = json.decodeFromString(response.body())
                        versionSchema shouldNotBe null
                    }
                }
            }
        }
    }

    afterProject {
        client.close()
    }
})
