package utils

import architectureUrl
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respondOk
import io.ktor.client.engine.mock.respondRedirect
import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import schemas.manifest.InstallerManifest.Installer.Architecture

class UrlUtilTests : FunSpec({
    context("get architecture from url") {
        val delimiters = listOf(",", ".", "_", "-", "/")

        context("x86 tests") {
            withData(
                "x86", "x32", "32-bit", "32bit", "Win32", "Winx86", "ia32", "i386", "386", "i486", "486", "i586", "586",
                "i686", "686"
            ) {
                withData(delimiters) { delimiter ->
                    architectureUrl(it, delimiter).findArchitecture() shouldBe Architecture.X86
                }
            }
        }

        context("x64 tests") {
            withData("x64", "x86_64", "64-bit", "64bit", "Win64", "Winx64", "amd64") {
                withData(delimiters) { delimiter ->
                    architectureUrl(it, delimiter).findArchitecture() shouldBe Architecture.X64
                }
            }
        }

        context("arm tests") {
            withData("arm", "aarch") {
                withData(delimiters) { delimiter ->
                    architectureUrl(it, delimiter).findArchitecture() shouldBe Architecture.ARM
                }
            }
        }

        context("arm64 tests") {
            withData("arm64", "aarch64") {
                withData(delimiters) { delimiter ->
                    architectureUrl(it, delimiter).findArchitecture() shouldBe Architecture.ARM64
                }
            }
        }

        test("architectures with no delimiters are not found") {
            architectureUrl(architecture = "armInstaller", delimiter = "").findArchitecture() shouldBe null
        }

        test("architecture at end of URL with extension") {
            architectureUrl(
                architecture = Architecture.ARM64,
                delimiterBefore = "",
                delimiterAfter = "."
            ).findArchitecture() shouldBe Architecture.ARM64
        }
    }

    context("get extension from url") {
        test("extension at end of url") {
            Url("example.com/fileName.exe").extension shouldBe "exe"
        }

        test("extension with /download before end of url") {
            Url("example.com/fileName.msi/download").extension shouldBe "msi"
        }
    }

    context("get file name without extension") {
        test("file at end of url") {
            Url("example.com/fileName.exe").getFileNameWithoutExtension() shouldBe "fileName"
        }

        test("file with /download before end of url") {
            Url("example.com/fileName.msi/download").getFileNameWithoutExtension() shouldBe "fileName"
        }
    }

    context("get scope from url") {
        test("user in url") {
            Url("example.com/fileName-user.exe").findScope() shouldBe InstallerManifest.Scope.User
        }

        test("machine in url") {
            Url("example.com/fileName-machine.exe").findScope() shouldBe InstallerManifest.Scope.Machine
        }
    }

    context("get redirected url") {
        val originalUrl = Url("firstWebsite/redirect")

        test("get redirected url when url has one redirect") {
            val newUrl = Url("newWebsite/fileName.exe")
            val mockEngine = MockEngine { request ->
                if (request.url == originalUrl) {
                    respondRedirect(newUrl.toString())
                } else {
                    respondOk("")
                }
            }
            HttpClient(mockEngine).use { originalUrl.getRedirectedUrl(it) shouldBe newUrl }
        }

        test("get redirected url when url has multiple redirects") {
            val intermediateUrl = Url("secondWebsite/redirect")
            val finalUrl = Url("newWebsite/fileName.exe")
            val mockEngine = MockEngine { request ->
                when (request.url) {
                    originalUrl -> respondRedirect(intermediateUrl.toString())
                    intermediateUrl -> respondRedirect(finalUrl.toString())
                    else -> respondOk("")
                }
            }
            HttpClient(mockEngine).use { originalUrl.getRedirectedUrl(it) shouldBe finalUrl }
        }

        test("get same url when url has url does not redirect") {
            val mockEngine = MockEngine { _ ->
                respondOk("")
            }
            HttpClient(mockEngine).use { originalUrl.getRedirectedUrl(it) shouldBe originalUrl }
        }

        test("url that keeps redirecting") {
            val mockEngine = MockEngine { _ ->
                respondRedirect(originalUrl.toString())
            }
            HttpClient(mockEngine).use { originalUrl.getRedirectedUrl(it) shouldBe originalUrl }
        }
    }

    context("check http status code redirects") {
        withData(
            listOf(
                HttpStatusCode.MultipleChoices,
                HttpStatusCode.MovedPermanently,
                HttpStatusCode.Found,
                HttpStatusCode.SeeOther,
                HttpStatusCode.NotModified,
                HttpStatusCode.UseProxy,
                HttpStatusCode.SwitchProxy,
                HttpStatusCode.TemporaryRedirect,
                HttpStatusCode.PermanentRedirect
            )
        ) {
            it.isRedirect shouldBe true
        }
    }
})
