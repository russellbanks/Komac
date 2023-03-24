package utils

import data.VersionUpdateState
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.should
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import io.kotest.matchers.string.shouldMatch
import io.kotest.matchers.string.startWith
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.delay
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GHTree
import org.kohsuke.github.GHTreeEntry
import java.io.IOException

class GitHubUtilsTests : FunSpec({
    context("get all versions tests") {
        val packageIdentifier = "Package.Identifier"
        val defaultBranchName = "main"

        lateinit var manifestsTree: GHTree
        lateinit var manifestsEntry: GHTreeEntry
        lateinit var baseTree: GHTree
        lateinit var repository: GHRepository

        beforeTest {
            manifestsTree = mockk {
                every { getEntry(any()) } answers { call ->
                    val entry = mockk<GHTreeEntry> {
                        every { path } returns if ((call.invocation.args.first() as String).length == 1) {
                            (call.invocation.args.first() as String).first().toString()
                        } else {
                            call.invocation.args.first() as String
                        }
                    }
                    every { entry.asTree() } returns this@mockk
                    every { entry.sha } returns ""
                    entry
                }
            }
            manifestsEntry = mockk()
            every { manifestsEntry.asTree() } returns manifestsTree
            baseTree = mockk()
            every { baseTree.getEntry("manifests") } returns manifestsEntry
            repository = mockk {
                every { defaultBranch } returns defaultBranchName
                every { getTree(defaultBranchName) } returns baseTree
            }
        }

        afterTest {
            clearAllMocks()
        }

        fun Map<String, String>.mapToMockedGHTreeEntry(): List<GHTreeEntry> {
            return map { (pathString, typeString) ->
                mockk {
                    every { path } returns pathString
                    every { type } returns typeString
                }
            }
        }

        test("latest version between two versions") {
            every { repository.getTreeRecursive(any(), 1).tree } returns mapOf(
                "1.2.3/file.yaml" to "blob",
                "1.2.4/file.yaml" to "blob"
            ).mapToMockedGHTreeEntry()
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe listOf("1.2.3", "1.2.4")
        }

        test(".validation files are filtered out") {
            every { repository.getTreeRecursive(any(), 1).tree } returns mapOf(
                ".validation" to "blob"
            ).mapToMockedGHTreeEntry()
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe null
        }

        test("sub packages are filtered out") {
            every { repository.getTreeRecursive(any(), 1).tree } returns mapOf(
                "1.2.3" to "tree",
                "1.2.3/file.yaml" to "blob",
                "subPackage/1.2.3" to "tree",
                "subPackage/1.2.3/file.yaml" to "blob"
            ).mapToMockedGHTreeEntry()
            GitHubUtils.getAllVersions(repository, packageIdentifier) shouldBe listOf("1.2.3")
        }

        test("IOException thrown") {
            every { repository.getTreeRecursive(any(), 1) } throws IOException()
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

    context("get commit title") {
        val identifier = "Package.Identifier"
        val version = "1.2.3"

        test("commit title for new version") {
            GitHubUtils.getCommitTitle(
                identifier,
                version,
                VersionUpdateState.NewVersion
            ) shouldBe "New version: $identifier version $version"
        }

        test("commit title for update version") {
            GitHubUtils.getCommitTitle(
                identifier,
                version,
                VersionUpdateState.UpdateVersion
            ) shouldBe "Update version: $identifier version $version"
        }

        test("commit title for new package") {
            GitHubUtils.getCommitTitle(
                identifier,
                version,
                VersionUpdateState.NewPackage
            ) shouldBe "New package: $identifier version $version"
        }

        test("commit title for add version") {
            GitHubUtils.getCommitTitle(
                identifier,
                version,
                VersionUpdateState.AddVersion
            ) shouldBe "Add version: $identifier version $version"
        }
    }

    context("get branch name") {
        val identifier = "Package.Identifier"
        val version = "1.2.3"

        test("generated branch name should start with package identifier and version") {
            GitHubUtils.getBranchName(identifier, version) should startWith("$identifier-$version-")
        }

        test("generated branch name should end with a unique identifier of length 32") {
            GitHubUtils.getBranchName(identifier, version) shouldMatch "^$identifier-$version-[A-Z0-9]{32}$"
        }

        test("generated branch name should be different each time it is generated") {
            val branchName1 = GitHubUtils.getBranchName(identifier, version)
            delay(1) // Wait 1 millisecond so that the timestamp is different
            val branchName2 = GitHubUtils.getBranchName(identifier, version)
            branchName1 shouldNotBe branchName2
        }
    }
})
