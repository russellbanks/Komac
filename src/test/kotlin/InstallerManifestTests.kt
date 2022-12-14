
import data.InstallerManifestChecks
import io.kotest.assertions.ktor.client.shouldHaveStatus
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.koin.KoinExtension
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
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
import org.koin.ksp.generated.defaultModule
import org.koin.test.KoinTest
import schemas.InstallerSchema
import schemas.LocaleSchema
import schemas.Schemas
import schemas.VersionSchema

class InstallerManifestTests : FunSpec(), KoinTest {
    override fun extensions() = listOf(KoinExtension(defaultModule))

    init {
        val client = HttpClient(Java) {
            install(UserAgent) {
                agent = "Microsoft-Delivery-Optimization/10.1"
            }
        }

        lateinit var installerSchema: InstallerSchema
        lateinit var localeSchema: LocaleSchema
        lateinit var versionSchema: VersionSchema

        listOf(
            Schemas.installerSchema,
            Schemas.localeSchema,
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
                        Schemas.localeSchema -> localeSchema = json.decodeFromString(response.body())
                        Schemas.versionSchema -> versionSchema = json.decodeFromString(response.body())
                    }
                }

                test("Validate parsed manifest") {
                    when (it) {
                        Schemas.installerSchema -> installerSchema.shouldNotBeNull()
                        Schemas.localeSchema -> localeSchema.shouldNotBeNull()
                        Schemas.versionSchema -> versionSchema.shouldNotBeNull()
                    }
                }
            }
        }

        context("Package Identifier Tests") {
            withData(
                listOf(
                    "ThisIsATest.Test",
                    "Test.test",
                    "test.test"
                )
            ) { identifier ->
                InstallerManifestChecks.isPackageIdentifierValid(identifier, installerSchema).first
                    .shouldBe(Validation.Success)
            }

            withData(
                listOf(
                    null,
                    "test",
                    ".",
                    "test./",
                    "test/test",
                )
            ) { identifier ->
                InstallerManifestChecks.isPackageIdentifierValid(identifier, installerSchema).first
                    .shouldNotBe(Validation.Success)
            }
        }

        context("Package Version Tests") {
            withData(
                listOf(
                    "1.0.0",
                    "1.2.3",
                    "1.1.1.1",
                    "1",
                    "v1",
                    "v1.0",
                    "v1.0.0",
                    "v1.0.0.0",
                    "v1."
                )
            ) { version ->
                InstallerManifestChecks.isPackageVersionValid(version, installerSchema).first
                    .shouldBe(Validation.Success)
            }

            withData(
                listOf(
                    null,
                    "/",
                    "?"
                )
            ) { version ->
                InstallerManifestChecks.isPackageVersionValid(version, installerSchema).first
                    .shouldNotBe(Validation.Success)
            }
        }

        context("Installer Url Tests") {
            withData(
                listOf("https://github.com")
            ) { url ->
                InstallerManifestChecks.isInstallerUrlValid(url, installerSchema).first.shouldBe(Validation.Success)
            }
        }

        context("Architecture Tests") {
            withData(
                listOf(
                    "x64",
                    "x86",
                    "arm",
                    "arm64",
                    "neutral"
                )
            ) {
                InstallerManifestChecks.isArchitectureValid(it, installerSchema).first.shouldBe(Validation.Success)
            }

            withData(
                listOf(
                    "64",
                    "86",
                    "x32",
                    "64bit",
                    "32bit",
                    "arm32",
                    "arm32bit",
                    "arm64bit",
                    "x64bit",
                    null
                )
            ) {
                InstallerManifestChecks.isArchitectureValid(it, installerSchema).first.shouldNotBe(Validation.Success)
            }
        }

        context("Installer Type Checks") {
            withData(
                listOf(
                    "msix",
                    "msi",
                    "appx",
                    "exe",
                    "zip",
                    "inno",
                    "nullsoft",
                    "wix",
                    "burn",
                    "pwa",
                    "portable"
                )
            ) {
                InstallerManifestChecks.isInstallerTypeValid(it, installerSchema).first.shouldBe(Validation.Success)
            }

            withData(
                listOf(
                    "msixx",
                    "appxx",
                    "exx",
                    "zipp",
                    "inn",
                    "nullsof",
                    "wixx",
                    "burnn",
                    "pwaa",
                    "portablee",
                    null
                )
            ) {
                InstallerManifestChecks.isInstallerTypeValid(it, installerSchema).first.shouldNotBe(Validation.Success)
            }
        }

        context("Installer Switch Checks") {
            InstallerSwitch.values().forEach { installerSwitch ->
                withData(
                    listOf(
                        "/S",
                        "-silent",
                        "/silent",
                        "-SILENT",
                        "/norestart",
                        "-norestart"
                    )
                ) {
                    InstallerManifestChecks.isInstallerSwitchValid(
                        switch = it,
                        installerSwitch = installerSwitch,
                        canBeBlank = false,
                        installerSchema = installerSchema
                    ).first.shouldBe(Validation.Success)
                }
            }
        }

        context("Installer Locale Checks") {
            withData(
                listOf(
                    "en-US",
                    "en-GB",
                    "en-CA",
                    "en-AU",
                )
            ) {
                InstallerManifestChecks.isInstallerLocaleValid(it, installerSchema).first.shouldBe(Validation.Success)
            }
        }

        afterProject {
            client.close()
        }
    }
}
