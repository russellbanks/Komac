package extensions

import extensions.IterableExtensions.takeIfNotDistinct
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.ktor.http.Url
import schemas.manifest.InstallerManifest

class IterableExtensionsTest : FunSpec({
    context("takeIfNotDistinct") {
        val baseInstaller = InstallerManifest.Installer(
            architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
            installerUrl = Url(""),
            installerSha256 = ""
        )
        val userScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Installer.Scope.User)
        val machineScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Installer.Scope.Machine)

        test("returns default value if iterable is not distinct") {
            listOf(
                userScopeInstaller,
                machineScopeInstaller
            ).takeIfNotDistinct(default = machineScopeInstaller.scope) { it.scope } shouldBe machineScopeInstaller.scope
        }

        test("returns null if iterable is distinct") {
            listOf(
                userScopeInstaller,
                userScopeInstaller,
                userScopeInstaller
            ).takeIfNotDistinct(default = userScopeInstaller.scope) { it.scope } shouldBe null
        }
    }
})
