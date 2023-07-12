
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import utils.UrlsToInstallerMatcher

class ParameterUrlTests : FunSpec({
    context("parameter url tests") {
        val baseInstaller = InstallerManifest.Installer(
            architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
            installerUrl = Url(""),
            installerSha256 = ""
        )

        /**
         * This is a test modelled off VSCodium.VSCodium, where half the installers are User scope and the other half
         * are Machine scope, but we only identify the User scopes.
         */
        test("VSCodium.VSCodium") {
            val installerX86 = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X86,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X86)
            )
            val installerUserX86 = installerX86.copy(scope = InstallerManifest.Scope.User)
            val installerX64 = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X64,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X64)
            )
            val installerUserX64 = installerX64.copy(scope = InstallerManifest.Scope.User)
            val previousInstallerMachineX86 = installerX86.copy(scope = InstallerManifest.Scope.Machine)
            val previousInstallerMachineX64 = installerX64.copy(scope = InstallerManifest.Scope.Machine)
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
            UrlsToInstallerMatcher.matchInstallers(newInstallers, previousInstallers) shouldBe mapOf(
                installerUserX86 to installerUserX86,
                previousInstallerMachineX86 to installerX86,
                installerUserX64 to installerUserX64,
                previousInstallerMachineX64 to installerX64
            )
        }

        /**
         * This is a test modelled off ArmCord.ArmCord, where the previous installers are ARM64 and X64, but we identify
         * the installers as ARM64 and X86.
         */
        test("ArmCord.ArmCord") {
            val previousArm64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.ARM64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                scope = InstallerManifest.Scope.User,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.ARM64),
            )
            val previousX64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                scope = InstallerManifest.Scope.User,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X64),
            )
            val newArm64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.ARM64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.ARM64),
            )
            val newX86Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X86,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(null),
            )
            val newInstallers = listOf(newArm64Installer, newX86Installer)
            val previousInstallers = listOf(previousArm64Installer, previousX64Installer)

            UrlsToInstallerMatcher.matchInstallers(newInstallers, previousInstallers) shouldBe mapOf(
                previousArm64Installer to newArm64Installer,
                previousX64Installer to newX86Installer
            )
        }

        /**
         * This is a test modelled off Fndroid.ClashForWindows, where the previous installers are ARM64, X64, and X86,
         * but the X64 installer has no architecture in its URL, and we identify it as X86.
         */
        test("Fndroid.ClashForWindows") {
            val previousArm64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.ARM64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.ARM64),
            )
            val previousX64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X64),
            )
            val previousX86Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X86,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X86),
            )
            val newArm64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.ARM64,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.ARM64),
            )
            val newX64Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X86,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(null),
            )
            val newX86Installer = baseInstaller.copy(
                architecture = InstallerManifest.Installer.Architecture.X86,
                installerType = InstallerManifest.InstallerType.NULLSOFT,
                installerUrl = architectureUrl(InstallerManifest.Installer.Architecture.X86),
            )

            val newInstallers = listOf(newArm64Installer, newX64Installer, newX86Installer)
            val previousInstallers = listOf(previousX86Installer, previousX64Installer, previousArm64Installer)

            UrlsToInstallerMatcher.matchInstallers(newInstallers, previousInstallers) shouldBe mapOf(
                previousX86Installer to newX86Installer,
                previousX64Installer to newX64Installer,
                previousArm64Installer to newArm64Installer
            )
        }

        test("empty map is returned if both newInstallers and previousInstallers are empty") {
            val newInstallers = emptyList<InstallerManifest.Installer>()
            val previousInstallers = emptyList<InstallerManifest.Installer>()
            UrlsToInstallerMatcher.matchInstallers(newInstallers, previousInstallers) shouldBe emptyMap()
        }
    }
})
