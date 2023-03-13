package data

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import com.russellbanks.Komac.BuildConfig
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
import schemas.Schemas
import utils.GitHubUtils
import java.io.IOException
import java.time.LocalDate
import kotlin.random.Random

class GitHubImpl(token: String, client: HttpClient) {
    val github: GitHub = GitHubBuilder().withConnector(KtorGitHubConnector(client)).withOAuthToken(token).build()
    private var pullRequestBranch: GHRef? = null

    private fun getBranchName(packageIdentifier: String, packageVersion: String): String {
        val randomPart = List(uniqueBranchIdentifierLength) { (('A'..'Z') + ('0'..'9')).random() }.joinToString("")
        return "$packageIdentifier-$packageVersion-$randomPart"
    }

    val forkOwner: String = System.getenv(customForkOwnerEnv) ?: github.myself.login

    fun getWingetPkgsFork(terminal: Terminal): GHRepository? = with(terminal) {
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
                danger(ioException.message ?: "Failed to fork $wingetpkgs. Please try again or fork it manually.")
                null
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
            createdSince = if (isCI) null else LocalDate.now().minusWeeks(2)
        ) ?: return
        warning("A pull request for $identifier $version was created on ${existingPullRequest.createdAt}")
        info(existingPullRequest.htmlUrl)
        if (isCI || YesNoPrompt("Would you like to proceed?", terminal = this).ask() != true) {
            throw ProgramResult(0)
        }
        println()
    }

    fun packageExists(identifier: String): Boolean? {
        return getMicrosoftWinGetPkgs()?.getDirectoryContent(GitHubUtils.getPackagePath(identifier))?.isNotEmpty()
    }

    fun versionExists(identifier: String, version: String): Boolean {
        return getMicrosoftWinGetPkgs()
            ?.getDirectoryContent(GitHubUtils.getPackagePath(identifier))
            ?.map(GHContent::getName)
            ?.contains(version)
            ?: false
    }

    fun getMicrosoftWinGetPkgs(): GHRepository? {
        return try {
            github.getRepository("$Microsoft/$wingetpkgs")
        } catch (_: IOException) {
            null
        }
    }

    fun createBranchFromUpstreamDefaultBranch(
        repository: GHRepository,
        packageIdentifier: String,
        packageVersion: String,
        terminal: Terminal
    ): GHRef? {
        return try {
            getMicrosoftWinGetPkgs()?.let { upstreamRepository ->
                repository.createRef(
                    /* name = */ "refs/heads/${getBranchName(packageIdentifier, packageVersion)}",
                    /* sha = */ upstreamRepository.getBranch(upstreamRepository.defaultBranch).shA1
                ).also { pullRequestBranch = it }
            }
        } catch (ioException: IOException) {
            terminal.danger(ioException.message ?: "Failed to create branch.")
            null
        }
    }

    private fun getCommitTitle(
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState
    ): String {
        return "$updateState: $packageIdentifier version $packageVersion"
    }

    private fun getPullRequestBody(): String {
        return buildString {
            append("### Pull request has been created with ")
            append(System.getenv(Schemas.customToolEnv) ?: "${BuildConfig.appName} v${BuildConfig.appVersion}")
            append(" ")
            append(if (Random.nextInt(30) == 0) ":${fruits[Random.nextInt(fruits.size)]}:" else ":rocket:")
        }
    }

    private val fruits = listOf(
        "cherries", "grapes", "green_apple", "lemon", "melon", "pineapple", "strawberry", "tangerine", "watermelon"
    )

    fun commitAndPullRequest(
        files: Map<String, String>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState,
        terminal: Terminal
    ): GHPullRequest? {
        commitFiles(
            files = files.mapKeys { "${GitHubUtils.getPackageVersionsPath(packageIdentifier, packageVersion)}/${it.key}" },
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            updateState = updateState,
            terminal = terminal
        )
        return createPullRequest(packageIdentifier, packageVersion, updateState)
    }

    private fun createPullRequest(
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState,
    ): GHPullRequest? {
        val ghRepository = getMicrosoftWinGetPkgs() ?: return null
        return try {
            ghRepository.createPullRequest(
                /* title = */ getCommitTitle(packageIdentifier, packageVersion, updateState),
                /* head = */ "$forkOwner:${pullRequestBranch?.ref}",
                /* base = */ ghRepository.defaultBranch,
                /* body = */ getPullRequestBody()
            )
        } catch (_: IOException) {
            null
        }
    }

    private fun commitFiles(
        files: Map<String, String?>,
        packageIdentifier: String,
        packageVersion: String,
        updateState: VersionUpdateState,
        terminal: Terminal
    ) {
        val repository = getWingetPkgsFork(terminal) ?: return
        val branch = createBranchFromUpstreamDefaultBranch(repository, packageIdentifier, packageVersion, terminal) ?: return
        repository.createCommit()
            ?.message(getCommitTitle(packageIdentifier, packageVersion, updateState))
            ?.parent(branch.getObject()?.sha)
            ?.tree(
                repository
                    .createTree()
                    .baseTree(repository.getBranch(branch.ref).shA1)
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
                        /* name = */ System.getenv("GIT_COMMITTER_NAME"),
                        /* email = */ System.getenv("GIT_COMMITTER_EMAIL"),
                        /* date = */ null
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
        private const val customForkOwnerEnv = "KMC_FRK_OWNER"
        private const val uniqueBranchIdentifierLength = 14
    }
}
