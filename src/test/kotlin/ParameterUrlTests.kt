
import detection.ParameterUrls
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.ktor.http.Url
import schemas.manifest.InstallerManifest

class ParameterUrlTests : FunSpec({
    context("parameter url tests") {
        test("match scoped installers where new installers only have user or null scope") {
            val baseInstaller = InstallerManifest.Installer(
                architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
                installerUrl = Url(""),
                installerSha256 = ""
            )
            val installerX86 = baseInstaller.copy(architecture = InstallerManifest.Installer.Architecture.X86)
            val installerUserX86 = installerX86.copy(scope = InstallerManifest.Installer.Scope.User)
            val installerX64 = baseInstaller.copy(architecture = InstallerManifest.Installer.Architecture.X64)
            val installerUserX64 = installerX64.copy(scope = InstallerManifest.Installer.Scope.User)
            val previousInstallerMachineX86 = installerX86.copy(scope = InstallerManifest.Installer.Scope.Machine)
            val previousInstallerMachineX64 = installerX64.copy(scope = InstallerManifest.Installer.Scope.Machine)
            val newInstallers = listOf(
                installerUserX86,
                installerX86,
                installerUserX64,
                installerX64
            )
            val previousInstallers = listOf(
                installerUserX86,
                previousInstallerMachineX86,
                installerUserX64,
                previousInstallerMachineX64
            )
            ParameterUrls.matchInstallers(newInstallers, previousInstallers) shouldBe mapOf(
                installerUserX86 to installerUserX86,
                previousInstallerMachineX86 to installerX86,
                installerUserX64 to installerUserX64,
                previousInstallerMachineX64 to installerX64
            )
        }

        test("empty map is returned if both newInstallers and previousInstallers are empty") {
            val newInstallers = emptyList<InstallerManifest.Installer>()
            val previousInstallers = emptyList<InstallerManifest.Installer>()
            ParameterUrls.matchInstallers(newInstallers, previousInstallers) shouldBe emptyMap()
        }
    }
})
