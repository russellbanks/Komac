package github

import Environment
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import data.VersionUpdateState
import java.io.IOException
import network.KtorGitHubConnector
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHDirection
import org.kohsuke.github.GHIssue
import org.kohsuke.github.GHIssueSearchBuilder
import org.kohsuke.github.GHIssueState
import org.kohsuke.github.GHPullRequest
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import token.TokenStore

object GitHubImpl {
    private const val Microsoft = "Microsoft"
    private const val wingetpkgs = "winget-pkgs"
    const val wingetPkgsFullName = "$Microsoft/$wingetpkgs"
    private const val customForkOwnerEnv = "KMC_FRK_OWNER"
    val github: GitHub = GitHubBuilder().withConnector(KtorGitHubConnector()).withOAuthToken(TokenStore.token).build()
    private var pullRequestBranch: GHRef? = null
    val forkOwner: String = System.getenv(customForkOwnerEnv) ?: github.myself.login

    val microsoftWinGetPkgs: GHRepository by lazy {
        var result: GHRepository? = null
        var count = 0
        val maxTries = 3
        while (result == null) {
            try {
                result = github.getRepository("$Microsoft/$wingetpkgs")
            } catch (ioException: IOException) {
                if (++count == maxTries) {
                    throw CliktError(
                        message = "Failed to get $wingetPkgsFullName",
                        cause = ioException,
                        statusCode = 1
                    )
                }
            }
        }
        result
    }

    fun getWingetPkgsFork(terminal: Terminal): GHRepository = with(terminal) {
        return try {
            github.getRepository("$forkOwner/$wingetpkgs")
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

    private fun getExistingPullRequest(identifier: String, version: String): GHIssue? = github.searchIssues()
        .q("repo:$Microsoft/$wingetpkgs")
        .q("is:pull-request")
        .q("in:title")
        .q(identifier)
        .q(version)
        .sort(GHIssueSearchBuilder.Sort.CREATED)
        .order(GHDirection.DESC)
        .list()
        .withPageSize(1)
        .firstOrNull()

    fun promptIfPullRequestExists(identifier: String, version: String, terminal: Terminal) = with(terminal) {
        val existingPullRequest = getExistingPullRequest(identifier, version) ?: return
        val isOpen = existingPullRequest.state == GHIssueState.OPEN
        warning(
            "There is already ${
                if (isOpen) "an open" else "a closed"
            } pull request for $identifier $version that was created on ${existingPullRequest.createdAt}"
        )
        info(existingPullRequest.htmlUrl)
        if (Environment.isCI) {
            if (isOpen) throw ProgramResult(0)
        } else {
            if (YesNoPrompt("Would you like to proceed?", terminal = this).ask() != true) throw ProgramResult(0)
        }
        println()
    }

    fun versionExists(identifier: String, version: String): Boolean = microsoftWinGetPkgs
        .getDirectoryContent(GitHubUtils.getPackagePath(identifier))
        ?.map(GHContent::getName)
        ?.contains(version) == true

    fun createBranchFromUpstreamDefaultBranch(
        winGetPkgsFork: GHRepository,
        packageIdentifier: String,
        packageVersion: String
    ): GHRef? {
        require(winGetPkgsFork.isFork)
        var count = 0
        val maxTries = 3
        while (true) {
            try {
                return winGetPkgsFork.source?.let { upstreamRepository ->
                    winGetPkgsFork.createRef(
                        "refs/heads/${GitHubUtils.getBranchName(packageIdentifier, packageVersion)}",
                        upstreamRepository.getBranch(upstreamRepository.defaultBranch).shA1
                    ).also { pullRequestBranch = it }
                }
            } catch (ioException: IOException) {
                if (++count >= maxTries) {
                    throw CliktError(
                        message = "Failed to create branch from upstream default branch",
                        cause = ioException,
                        statusCode = 1
                    )
                }
            }
        }
    }


    fun commitAndPullRequest(
        wingetPkgsFork: GHRepository,
        files: Map<String, String>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState
    ): GHPullRequest {
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
    ): GHPullRequest {
        val ghRepository = microsoftWinGetPkgs
        var count = 0
        val maxTries = 3
        while (true) {
            try {
                return ghRepository.createPullRequest(
                    /* title = */ GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState),
                    /* head = */ "$forkOwner:${pullRequestBranch?.ref}",
                    /* base = */ ghRepository.defaultBranch,
                    /* body = */ GitHubUtils.getPullRequestBody()
                )
            } catch (ioException: IOException) {
                if (++count >= maxTries) {
                    throw CliktError(
                        message = """
                            Failed to create pull request after $maxTries attempts.
                            ${ioException.message?.let { "Reason: $it" }}.
                        """.trimIndent(),
                        cause = ioException,
                        statusCode = 1
                    )
                }
            }
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
                        for ((path, content) in files) {
                            if (content != null) {
                                add(path, content.replace("\n", "\r\n"), false)
                            }
                        }
                    }
                    .create()
                    .sha
            )
            ?.create()
            ?.also { branch.updateTo(it.shA1) }
    }
}
