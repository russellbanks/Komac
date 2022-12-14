
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

        context("Product Code Checks") {
            withData(
                listOf(
                    "{E1E861BE-3251-44C5-B927-80CA66D8727E}",
                    "{AE4D7451-6C83-4FEF-AC09-C72D0BAAE8F6}",
                    "{B894AEBC-EFBB-4109-8322-FCDCF1C72F23}",
                    "{40F1B3A4-38C4-4400-B7A2-3CDCF43DE2A2}",
                    "{943025A8-6863-4768-ADC5-A632E31A2B98}",
                    "{D0A2ECFA-0DD0-4827-8682-946DDCA9AFAA}",
                    "{9AFB4EB8-CDAA-4A25-9A2F-5040792C53E5}"
                )
            ) {
                InstallerManifestChecks.isProductCodeValid(it, installerSchema).first.shouldBe(Validation.Success)
            }
        }

        context("Installer Scope Tests") {
            withData(listOf('M', 'U')) {
                InstallerManifestChecks.isInstallerScopeValid(it, installerSchema).first.shouldBe(Validation.Success)
            }
        }

        context("Upgrade Behaviour Tests") {
            withData(listOf('I', 'U')) {
                InstallerManifestChecks.isUpgradeBehaviourValid(it, installerSchema).first.shouldBe(Validation.Success)
            }
        }

        context("Release Date Tests") {
            withData(listOf("2020-01-01", "2020-01-01")) {
                InstallerManifestChecks.isReleaseDateValid(it).first.shouldBe(Validation.Success)
            }

            withData(
                listOf(
                    "2022-13-01",
                    "2020-01-32",
                    "2020-01-01T00:00:00Z",
                    "2020-01-01T00:00:00+00:00",
                )
            ) {
                InstallerManifestChecks.isReleaseDateValid(it).first.shouldNotBe(Validation.Success)
            }
        }

        afterProject {
            client.close()
        }
    }
}
