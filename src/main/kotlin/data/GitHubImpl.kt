package data

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import io.ktor.client.HttpClient
import network.KtorGitHubConnector
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHDirection
import org.kohsuke.github.GHIssue
import org.kohsuke.github.GHIssueSearchBuilder
import org.kohsuke.github.GHPullRequest
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import utils.GitHubUtils
import java.io.IOException
import java.time.LocalDate

class GitHubImpl(token: String, client: HttpClient) {
    val github: GitHub = GitHubBuilder().withConnector(KtorGitHubConnector(client)).withOAuthToken(token).build()
    private var pullRequestBranch: GHRef? = null
    val forkOwner: String = System.getenv(customForkOwnerEnv) ?: github.myself.login

    fun getWingetPkgsFork(terminal: Terminal): GHRepository = with(terminal) {
        return try {
            github.getRepository("$forkOwner/$wingetpkgs").also {
                success("Found forked $wingetpkgs repository: ${it.fullName}")
            }
        } catch (_: IOException) {
            info("Fork of $wingetpkgs not found. Forking...")
            try {
                github.getRepository("$Microsoft/$wingetpkgs").fork().also {
                    success("Forked $wingetpkgs repository: ${it.fullName}")
                }
            } catch (ioException: IOException) {
                throw CliktError(
                    message = colors.danger("Failed to fork $wingetpkgs. Please try again or fork it manually"),
                    cause = ioException,
                    statusCode = 1
                )
            }
        }
    }

    private fun getExistingPullRequest(identifier: String, version: String, createdSince: LocalDate? = null): GHIssue? {
        return github.searchIssues()
            .q("repo:$Microsoft/$wingetpkgs")
            .q("is:pr")
            .q(identifier)
            .q(version)
            .q("in:path")
            .let { if (createdSince != null) it.q("created:>$createdSince") else it }
            .sort(GHIssueSearchBuilder.Sort.CREATED)
            .order(GHDirection.DESC)
            .list()
            .withPageSize(1)
            .firstOrNull()
    }

    fun promptIfPullRequestExists(identifier: String, version: String, terminal: Terminal) = with(terminal) {
        val isCI = System.getenv("CI")?.toBooleanStrictOrNull() == true
        val existingPullRequest = getExistingPullRequest(
            identifier = identifier,
            version = version,
            createdSince = if (isCI) null else LocalDate.now().minusWeeks(WEEKS_SINCE_CREATED)
        ) ?: return
        warning("A pull request for $identifier $version was created on ${existingPullRequest.createdAt}")
        info(existingPullRequest.htmlUrl)
        if (!isCI && YesNoPrompt("Would you like to proceed?", terminal = this).ask() != true) {
            throw ProgramResult(0)
        }
        println()
    }

    fun versionExists(identifier: String, version: String): Boolean {
        return getMicrosoftWinGetPkgs()
            .getDirectoryContent(GitHubUtils.getPackagePath(identifier))
            ?.map(GHContent::getName)
            ?.contains(version) == true
    }

    fun getMicrosoftWinGetPkgs(): GHRepository {
        var count = 0
        val maxTries = 3
        while (true) {
            try {
                return github.getRepository("$Microsoft/$wingetpkgs")
            } catch (ioException: IOException) {
                if (++count == maxTries) {
                    throw CliktError(message = "Failed to get $wingetPkgsFullName", cause = ioException, statusCode = 1)
                }
            }
        }
    }

    fun createBranchFromUpstreamDefaultBranch(
        wingetPkgsFork: GHRepository,
        packageIdentifier: String,
        packageVersion: String
    ): GHRef? {
        require(wingetPkgsFork.isFork)
        return try {
            wingetPkgsFork.source?.let { upstreamRepository ->
                wingetPkgsFork.createRef(
                    "refs/heads/${GitHubUtils.getBranchName(packageIdentifier, packageVersion)}",
                    upstreamRepository.getBranch(upstreamRepository.defaultBranch).shA1
                ).also { pullRequestBranch = it }
            }
        } catch (_: IOException) {
            null
        }
    }

    fun commitAndPullRequest(
        wingetPkgsFork: GHRepository,
        files: Map<String, String>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState
    ): GHPullRequest? {
        commitFiles(
            wingetPkgsFork = wingetPkgsFork,
            files = files.mapKeys { "${GitHubUtils.getPackageVersionsPath(packageIdentifier, packageVersion)}/${it.key}" },
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            updateState = updateState
        )
        return createPullRequest(packageIdentifier, packageVersion, updateState)
    }

    private fun createPullRequest(
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState,
    ): GHPullRequest? {
        val ghRepository = getMicrosoftWinGetPkgs()
        return try {
            ghRepository.createPullRequest(
                /* title = */ GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState),
                /* head = */ "$forkOwner:${pullRequestBranch?.ref}",
                /* base = */ ghRepository.defaultBranch,
                /* body = */ GitHubUtils.getPullRequestBody()
            )
        } catch (_: IOException) {
            null
        }
    }

    private fun commitFiles(
        wingetPkgsFork: GHRepository,
        files: Map<String, String?>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState
    ) {
        val branch = createBranchFromUpstreamDefaultBranch(wingetPkgsFork, packageIdentifier, packageVersion) ?: return
        wingetPkgsFork.createCommit()
            ?.message(GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState))
            ?.parent(branch.getObject()?.sha)
            ?.tree(
                wingetPkgsFork
                    .createTree()
                    .baseTree(wingetPkgsFork.getBranch(branch.ref).shA1)
                    .apply {
                        files.forEach { (path, content) ->
                            if (content != null) {
                                add(path, content.replace("\n", "\r\n"), false)
                            }
                        }
                    }
                    .create()
                    .sha
            )
            ?.let {
                if (System.getenv("GIT_COMMITTER_NAME") != null && System.getenv("GIT_COMMITTER_EMAIL") != null) {
                    it.author(
                        System.getenv("GIT_COMMITTER_NAME"),
                        System.getenv("GIT_COMMITTER_EMAIL"),
                        null
                    )
                } else {
                    it
                }
            }
            ?.create()
            ?.also { branch.updateTo(it.shA1) }
    }

    companion object {
        const val Microsoft = "Microsoft"
        const val wingetpkgs = "winget-pkgs"
        const val wingetPkgsFullName = "$Microsoft/$wingetpkgs"
        private const val WEEKS_SINCE_CREATED = 2L
        private const val customForkOwnerEnv = "KMC_FRK_OWNER"
    }
}
