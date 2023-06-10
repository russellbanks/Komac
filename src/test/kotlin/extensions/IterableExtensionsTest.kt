package extensions

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import utils.takeIfNotDistinct

class IterableExtensionsTest : FunSpec({
    context("takeIfNotDistinct") {
        val baseInstaller = InstallerManifest.Installer(
            architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
            installerUrl = Url(URLBuilder()),
            installerSha256 = ""
        )
        val userScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Scope.User)
        val machineScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Scope.Machine)

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
