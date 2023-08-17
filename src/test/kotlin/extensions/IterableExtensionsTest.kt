package extensions

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import utils.filterSingleDistinctOrElse
import utils.mapDistinctSingleOrNull

class IterableExtensionsTest : FunSpec({
    val baseInstaller = InstallerManifest.Installer(
        architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
        installerUrl = Url(""),
        installerSha256 = ""
    )
    val userScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Scope.User)
    val machineScopeInstaller = baseInstaller.copy(scope = InstallerManifest.Scope.Machine)

    test("filterSingleDistinctOrElse returns the default value if the iterable is not distinct by the mapper") {
        listOf(
            userScopeInstaller,
            machineScopeInstaller
        ).filterSingleDistinctOrElse(
            default = machineScopeInstaller.scope,
            selector = InstallerManifest.Installer::scope
        ) shouldBe machineScopeInstaller.scope
    }

    test("filterSingleDistinctOrElse returns null if the iterable is distinct by the mapper") {
        listOf(
            userScopeInstaller,
            userScopeInstaller,
            userScopeInstaller
        ).filterSingleDistinctOrElse(
            default = userScopeInstaller.scope,
            selector = InstallerManifest.Installer::scope
        ) shouldBe null
    }

    test("mapDistinctSingleOrNull returns null if the iterable is not distinct by the mapper") {
        listOf(
            userScopeInstaller,
            baseInstaller
        ).mapDistinctSingleOrNull(InstallerManifest.Installer::scope) shouldBe null
    }

    test("mapDistinctSingleOrNull returns the value if the iterable is distinct by the mapper") {
        listOf(
            userScopeInstaller,
            userScopeInstaller
        ).mapDistinctSingleOrNull(InstallerManifest.Installer::scope) shouldBe userScopeInstaller.scope
    }
})
