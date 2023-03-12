
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import utils.GitHubUtils

class GitHubUtilsTests : FunSpec({
    context("get all versions tests") {
        val packageIdentifier = "Package.Identifier"
        val repository: GHRepository = mockk()
        val directoryOne: GHContent = mockk()
        val directoryTwo: GHContent = mockk()

        test("latest version between two versions") {
            every { directoryOne.name } returns "1.2.3"
            every { directoryOne.isDirectory } returns true
            every { directoryTwo.name } returns "1.2.4"
            every { directoryTwo.isDirectory } returns true
            every {
                repository.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
            } returns listOf(directoryOne, directoryTwo)
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe listOf("1.2.3", "1.2.4")
        }

        test(".validation files are filtered out") {
            every { directoryOne.name } returns ".validation"
            every { directoryOne.isDirectory } returns false
            every {
                repository.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
            } returns listOf(directoryOne)
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe null
        }

        test("sub packages are filtered out") {
            every { directoryOne.name } returns "SubPackage"
            every { directoryOne.isDirectory } returns true
            every {
                repository.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
            } returns listOf(directoryOne)
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe null
        }
    }

    context("get manifest names") {
        test("get installer manifest name") {
            GitHubUtils.getInstallerManifestName("Package.Identifier") shouldBe "Package.Identifier.installer.yaml"
        }

        test("get default locale manifest name") {
            GitHubUtils.getDefaultLocaleManifestName(
                identifier = "Package.Identifier",
                defaultLocale = "en-US",
                previousDefaultLocale = "en-US"
            ) shouldBe "Package.Identifier.locale.en-US.yaml"
        }

        test("get locale manifest name") {
            GitHubUtils.getLocaleManifestName(
                identifier = "Package.Identifier",
                locale = "en-CA"
            ) shouldBe "Package.Identifier.locale.en-CA.yaml"
        }

        test("get version manifest name") {
            GitHubUtils.getVersionManifestName("Package.Identifier") shouldBe "Package.Identifier.yaml"
        }
    }

    context("get manifest paths") {
        test("get package path") {
            GitHubUtils.getPackagePath("Package.Identifier") shouldBe "manifests/p/Package/Identifier"
        }

        test("get package versions path") {
            GitHubUtils.getPackageVersionsPath(
                identifier = "Package.Identifier",
                version = "1.2.3"
            ) shouldBe "manifests/p/Package/Identifier/1.2.3"
        }
    }
})
