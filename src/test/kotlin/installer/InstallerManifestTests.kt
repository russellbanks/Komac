package installer
import Ktor
import Validation
import data.InstallerScope.isInstallerScopeValid
import data.UpgradeBehaviour.isUpgradeBehaviourValid
import io.kotest.assertions.ktor.client.shouldHaveStatus
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpStatusCode
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.koin.core.context.startKoin
import org.koin.ksp.generated.defaultModule
import schemas.DefaultLocaleSchema
import schemas.InstallerSchema
import schemas.Schemas
import schemas.VersionSchema

class InstallerManifestTests : FunSpec() {

    init {
        startKoin { defaultModule() }
        val client = HttpClient(Java) {
            install(UserAgent) {
                agent = Ktor.userAgent
            }
        }

        lateinit var installerSchema: InstallerSchema
        lateinit var defaultLocaleSchema: DefaultLocaleSchema
        lateinit var versionSchema: VersionSchema

        listOf(
            Schemas.installerSchema,
            Schemas.defaultLocaleSchema,
            Schemas.versionSchema
        ).forEach {
            context("Get $it") {
                lateinit var response: HttpResponse

                test("Retrieve $it") {
                    response = client.get(it)
                    with(response) {
                        shouldNotBeNull()
                        shouldHaveStatus(HttpStatusCode.OK)
                    }
                }

                test("Parse $it") {
                    val json = Json { ignoreUnknownKeys = true }
                    when (it) {
                        Schemas.installerSchema -> installerSchema = json.decodeFromString(response.body())
                        Schemas.defaultLocaleSchema -> defaultLocaleSchema = json.decodeFromString(response.body())
                        Schemas.versionSchema -> versionSchema = json.decodeFromString(response.body())
                    }
                }

                test("Validate parsed manifest") {
                    when (it) {
                        Schemas.installerSchema -> installerSchema.shouldNotBeNull()
                        Schemas.defaultLocaleSchema -> defaultLocaleSchema.shouldNotBeNull()
                        Schemas.versionSchema -> versionSchema.shouldNotBeNull()
                    }
                }
            }
        }

        context("Installer Scope Tests") {
            withData(listOf('M', 'U')) {
                isInstallerScopeValid(it, installerSchema.definitions.scope).first shouldBe Validation.Success
            }
        }

        context("Upgrade Behaviour Tests") {
            withData(listOf('I', 'U')) {
                isUpgradeBehaviourValid(
                    it,
                    installerSchema.definitions.upgradeBehavior
                ).first shouldBe Validation.Success
            }
        }

        afterProject {
            client.close()
        }
    }
}
