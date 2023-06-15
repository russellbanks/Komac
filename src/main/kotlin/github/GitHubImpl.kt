package github

import Environment
import Errors
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.VersionUpdateState
import io.ktor.client.request.headers
import io.ktor.client.request.patch
import io.ktor.client.request.setBody
import io.ktor.http.HttpHeaders
import io.menu.yesNoMenu
import java.io.IOException
import network.Http
import network.KtorGitHubConnector
import org.kohsuke.github.GHDirection
import org.kohsuke.github.GHIssue
import org.kohsuke.github.GHIssueSearchBuilder
import org.kohsuke.github.GHIssueState
import org.kohsuke.github.GHPullRequest
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.Manifest
import schemas.manifest.VersionManifest
import token.TokenStore

object GitHubImpl {
    private const val Microsoft = "Microsoft"
    private const val wingetpkgs = "winget-pkgs"
    const val wingetPkgsFullName = "$Microsoft/$wingetpkgs"
    val github: GitHub = GitHubBuilder().withConnector(KtorGitHubConnector()).withOAuthToken(TokenStore.token).build()
    private var pullRequestBranch: GHRef? = null
    val forkOwner: String = Environment.forkOverride ?: github.myself.login
    private val draftPullRequest = getDraftPullRequest()
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
                        message = "Failed to get $wingetPkgsFullName", cause = ioException, statusCode = 1
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

    private fun getDraftPullRequest(): GHPullRequest? = github.searchIssues()
        .q("repo:$Microsoft/$wingetpkgs")
        .q("is:pr")
        .q("draft:true")
        .q("author:${github.myself.login}")
        .isOpen()
        .list()
        .withPageSize(1)
        .firstOrNull()
        ?.let { microsoftWinGetPkgs.getPullRequest(it.number) }

    private fun updateExistingBranchToUpstreamDefaultBranch(
        wingetPkgsFork: GHRepository, branchName: String
    ): GHRef {
        val branch = wingetPkgsFork.getRef("heads/$branchName")
        val upstreamDefaultBranch = microsoftWinGetPkgs.getBranch(microsoftWinGetPkgs.defaultBranch)
        branch.updateTo(upstreamDefaultBranch.shA1)
        return branch
    }

    private fun getExistingPullRequest(identifier: String, version: String): GHIssue? = github.searchIssues()
        .q("repo:$Microsoft/$wingetpkgs")
        .q("is:pull-request")
        .q("in:title").q(identifier)
        .q(version)
        .sort(GHIssueSearchBuilder.Sort.CREATED)
        .order(GHDirection.DESC)
        .list()
        .withPageSize(1)
        .firstOrNull()

    fun Terminal.promptIfPullRequestExists(identifier: String, version: String) {
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
            info("Would you like to proceed?")
            if (!yesNoMenu(default = false).prompt()) throw ProgramResult(0)
        }
        println()
    }


    fun createBranchFromUpstreamDefaultBranch(
        winGetPkgsFork: GHRepository, packageIdentifier: String, packageVersion: String
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

    suspend fun commitAndPullRequest(
        wingetPkgsFork: GHRepository,
        files: Map<String, Manifest>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState,
        previousManifestData: PreviousManifestData,
        terminal: Terminal
    ): GHPullRequest {
        val manifests = files.values
        if (
            manifests.find { it is InstallerManifest } == previousManifestData.installerManifest &&
            manifests.find { it is DefaultLocaleManifest } == previousManifestData.defaultLocaleManifest &&
            manifests.find { it is VersionManifest } == previousManifestData.versionManifest &&
            manifests.filterIsInstance<LocaleManifest>() == previousManifestData.localeManifests
        ) {
            if (Environment.isCI) {
                throw CliktError(
                    message = Errors.noManifestChanges,
                    cause = null,
                    statusCode = 0 // Nothing went wrong, but we should still avoid making a pull request
                )
            } else {
                terminal.warning(Errors.noManifestChanges)
                terminal.info("Do you want to create a pull request anyway?")
                if (!terminal.yesNoMenu(default = false).prompt()) throw ProgramResult(0)
            }
        }
        commitFiles(
            wingetPkgsFork = wingetPkgsFork, files = files.mapKeys {
                "${
                    GitHubUtils.getPackageVersionsPath(
                        packageIdentifier, packageVersion
                    )
                }/${it.key}"
            }, packageIdentifier = packageIdentifier, packageVersion = packageVersion, updateState = updateState
        )
        if (Environment.forcePushOnDraftPR && draftPullRequest != null) {
            Http.client.patch("https://api.github.com/repos/$Microsoft/$wingetpkgs/pulls/${draftPullRequest.number}") {
                setBody(
                    """
                        {
                            "title": "${GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState)}",
                            "body": "${GitHubUtils.getPullRequestBody()}",
                            "draft": false
                        }
                    """.trimIndent()
                )
                headers {
                    append(HttpHeaders.Authorization, "token ${TokenStore.token}")
                    append(HttpHeaders.Accept, "application/vnd.github.v3+json")
                }
            }
            terminal.info("Updated existing draft pull request")
            terminal.info(draftPullRequest.htmlUrl)
        }
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
                    GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState),
                    "$forkOwner:${pullRequestBranch?.ref}",
                    ghRepository.defaultBranch,
                    GitHubUtils.getPullRequestBody()
                )
            } catch (ioException: IOException) {
                if (++count >= maxTries) {
                    throw CliktError(
                        message = """
                                Failed to create pull request after $maxTries attempts .
                    ${ioException.message?.let { "Reason: $it" }}.""".trimIndent(), cause = ioException, statusCode = 1
                    )
                }
            }
        }
    }

    private fun commitFiles(
        wingetPkgsFork: GHRepository,
        files: Map<String, Manifest?>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState
    ) {
        val branch = if (Environment.forcePushOnDraftPR && draftPullRequest != null) {
            updateExistingBranchToUpstreamDefaultBranch(
                wingetPkgsFork, microsoftWinGetPkgs.getPullRequest(draftPullRequest.number).head.ref
            )
        } else {
            createBranchFromUpstreamDefaultBranch(wingetPkgsFork, packageIdentifier, packageVersion)
        }
        wingetPkgsFork.createCommit()
            ?.message(GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, updateState))
            ?.parent(branch?.getObject()?.sha)
            ?.tree(
                wingetPkgsFork.createTree()
                    .baseTree(wingetPkgsFork.getBranch(branch?.ref).shA1)
                    .apply {
                        for ((path, content) in files) {
                            if (content != null) {
                                add(path, content.toString().replace("\n", "\r\n"), false)
                            }
                        }
                    }
                    .create()
                    .sha
            )
            ?.create()
            ?.also { branch?.updateTo(it.shA1) }
    }
}
