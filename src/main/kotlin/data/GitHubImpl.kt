package data

import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import com.russellbanks.Komac.BuildConfig
import network.Http
import network.KtorGitHubConnector
import org.kohsuke.github.GHDirection
import org.kohsuke.github.GHIssue
import org.kohsuke.github.GHIssueSearchBuilder
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schemas
import token.TokenStore
import java.io.IOException
import java.time.LocalDate
import kotlin.random.Random
import kotlin.system.exitProcess

@Single
class GitHubImpl : KoinComponent {
    val github: GitHub = GitHubBuilder()
        .withConnector(KtorGitHubConnector(get<Http>().client))
        .withOAuthToken(get<TokenStore>().token)
        .build()
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    val installerManifestName = "${allManifestData.packageIdentifier}.installer.yaml"
    val versionManifestName = "${allManifestData.packageIdentifier}.yaml"
    private var pullRequestBranch: GHRef? = null

    suspend fun getDefaultLocaleManifestName() = buildString {
        append(allManifestData.packageIdentifier)
        append(".locale.")
        append(allManifestData.defaultLocale ?: previousManifestData.remoteVersionData.await()?.defaultLocale!!)
        append(".yaml")
    }

    val packageVersionsPath
        get() = buildString {
            append("manifests/")
            append("${allManifestData.packageIdentifier.first().lowercase()}/")
            append(allManifestData.packageIdentifier.replace(".", "/"))
        }

    val baseGitHubPath
        get() = "$packageVersionsPath/${allManifestData.packageVersion}"

    fun getLocaleManifestName(locale: String): String {
        return "${allManifestData.packageIdentifier}.locale.$locale.yaml"
    }

    private fun getBranchName() = buildString {
        append(allManifestData.packageIdentifier)
        append("-")
        append(allManifestData.packageVersion)
        append("-")
        append(List(uniqueBranchIdentifierLength) { (('A'..'Z') + ('0'..'9')).random() }.joinToString(""))
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

    private fun getExistingPullRequest(identifier: String, version: String): GHIssue? {
        return github.searchIssues()
            .q("repo:$Microsoft/$wingetpkgs")
            .q("is:pr")
            .q(identifier)
            .q(version)
            .q("in:path")
            .q("created:>${LocalDate.now().minusWeeks(2)}")
            .sort(GHIssueSearchBuilder.Sort.CREATED)
            .order(GHDirection.DESC)
            .list()
            .withPageSize(1)
            .firstOrNull()
    }

    fun promptIfPullRequestExists(identifier: String, version: String, terminal: Terminal) = with(terminal) {
        val ghIssue = getExistingPullRequest(identifier, version) ?: return
        warning("A pull request for $identifier $version was created on ${ghIssue.createdAt}")
        info(ghIssue.htmlUrl)
        if (
            System.getenv("CI")?.toBooleanStrictOrNull() != true &&
            YesNoPrompt("Would you like to proceed?", terminal = this).ask() != true
        ) {
            exitProcess(0)
        }
        println()
    }

    fun getMicrosoftWinGetPkgs(): GHRepository? {
        return try {
            github.getRepository("$Microsoft/$wingetpkgs")
        } catch (_: IOException) {
            null
        }
    }

    fun createBranchFromUpstreamDefaultBranch(repository: GHRepository, terminal: Terminal): GHRef? {
        return try {
            getMicrosoftWinGetPkgs()?.let { upstreamRepository ->
                repository.createRef(
                    /* name = */ "refs/heads/${getBranchName()}",
                    /* sha = */ upstreamRepository.getBranch(upstreamRepository.defaultBranch).shA1
                ).also { pullRequestBranch = it }
            }
        } catch (ioException: IOException) {
            terminal.danger(ioException.message ?: "Failed to create branch.")
            null
        }
    }

    private fun getCommitTitle(
        updateState: VersionUpdateState,
        packageIdentifier: String,
        packageVersion: String
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

    fun commitAndPullRequest(files: List<Pair<String, String>>, terminal: Terminal) {
        commitFiles(files.map { "$baseGitHubPath/${it.first}" to it.second }, terminal)
        createPullRequest(terminal)
    }

    private fun createPullRequest(terminal: Terminal): Unit = with(allManifestData) {
        val ghRepository = getMicrosoftWinGetPkgs() ?: return
        try {
            ghRepository.createPullRequest(
                /* title = */ getCommitTitle(updateState, packageIdentifier, packageVersion),
                /* head = */ "$forkOwner:${pullRequestBranch?.ref}",
                /* base = */ ghRepository.defaultBranch,
                /* body = */ getPullRequestBody()
            ).also { terminal.success("Pull request created: ${it.htmlUrl}") }
        } catch (ioException: IOException) {
            terminal.danger(ioException.message ?: "Failed to create pull request")
        }
    }

    private fun commitFiles(files: List<Pair<String, String?>>, terminal: Terminal): Unit = with(allManifestData) {
        val repository = getWingetPkgsFork(terminal) ?: return
        val branch = createBranchFromUpstreamDefaultBranch(repository, terminal) ?: return
        repository.createCommit()
            ?.message(getCommitTitle(updateState, packageIdentifier, packageVersion))
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
