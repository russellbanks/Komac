
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.ktor.http.Url
import network.HttpUtils.detectArchitectureFromUrl
import schemas.manifest.InstallerManifest.Installer.Architecture

class UrlUtilTests : FunSpec({
    context("get architecture from url") {
        fun architectureUrl(architecture: String) = Url("file-$architecture.extension")

        context("x86 tests") {
            withData("x86", "i386", "386", "i486", "486", "i586", "586", "i686", "686") {
                detectArchitectureFromUrl(architectureUrl(it)) shouldBe Architecture.X86
            }
        }

        context("x64 tests") {
            withData("x64", "x86_64", "amd64") {
                detectArchitectureFromUrl(architectureUrl(it)) shouldBe Architecture.X64
            }
        }

        context("arm tests") {
            withData("arm", "aarch") {
                detectArchitectureFromUrl(architectureUrl(it)) shouldBe Architecture.ARM
            }
        }

        context("arm64 tests") {
            withData("arm64", "aarch64") {
                detectArchitectureFromUrl(architectureUrl(it)) shouldBe Architecture.ARM64
            }
        }
    }
})
