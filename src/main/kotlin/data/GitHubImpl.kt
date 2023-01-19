package data

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.terminal.Terminal
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import ktor.Clients
import ktor.KtorGitHubConnector
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import token.TokenStore
import java.io.IOException

@Single
class GitHubImpl : KoinComponent {
    val github: Deferred<GitHub> = CoroutineScope(Dispatchers.IO).async {
        GitHubBuilder()
            .withConnector(KtorGitHubConnector(get<Clients>().httpClient))
            .withOAuthToken(get<TokenStore>().token.await())
            .build()
    }
    private val sharedManifestData: SharedManifestData by inject()
    val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    val defaultLocaleManifestName
        get() = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    val versionManifestName = "${sharedManifestData.packageIdentifier}.yaml"
    var pullRequestBranch: GHRef? = null

    val packageVersionsPath
        get() = buildString {
            append("manifests/")
            append("${sharedManifestData.packageIdentifier.first().lowercase()}/")
            append(sharedManifestData.packageIdentifier.replace(".", "/"))
        }

    val baseGitHubPath
        get() = "$packageVersionsPath/${sharedManifestData.packageVersion}"

    fun getLocaleManifestName(locale: String): String {
        return "${sharedManifestData.packageIdentifier}.locale.$locale.yaml"
    }

    private val branchName: String
        get() = buildString {
            append(sharedManifestData.packageIdentifier)
            append("-")
            append(sharedManifestData.packageVersion)
            append("-")
            append(List(uniqueBranchIdentifierLength) { (('A'..'Z') + ('0'..'9')).random() }.joinToString(""))
        }

    suspend fun getWingetPkgsFork(terminal: Terminal): GHRepository? {
        with(terminal) {
            return try {
                github.await().getRepository("${github.await().myself.login}/$wingetpkgs").also {
                    println(brightWhite("Found forked $wingetpkgs repository: ${it.fullName}"))
                }
            } catch (_: IOException) {
                println(brightWhite("Fork of $wingetpkgs not found. Forking..."))
                try {
                    github.await().getRepository("$Microsoft/$wingetpkgs").fork().also {
                        println(brightGreen("Forked $wingetpkgs repository: ${it.fullName}"))
                    }
                } catch (ioException: IOException) {
                    println(
                        brightRed(
                            ioException.message ?: "Failed to fork $wingetpkgs. Please try again or fork it manually."
                        )
                    )
                    null
                }
            }
        }
    }

    suspend fun getMicrosoftWingetPkgs(): GHRepository? {
        return try {
            github.await().getRepository("$Microsoft/$wingetpkgs")
        } catch (_: IOException) {
            null
        }
    }

    fun createBranchFromDefaultBranch(repository: GHRepository, terminal: Terminal): GHRef? {
        return try {
            repository.createRef(
                /* name = */ "refs/heads/$branchName",
                /* sha = */ repository.getBranch(repository.defaultBranch).shA1
            ).also { pullRequestBranch = it }
        } catch (ioException: IOException) {
            terminal.println(brightRed(ioException.message ?: "Failed to create branch."))
            null
        }
    }

    fun getCommitTitle() = buildString {
        append(sharedManifestData.updateState)
        append(": ")
        append(sharedManifestData.packageIdentifier)
        append(" version ")
        append(sharedManifestData.packageVersion)
    }

    fun getPullRequestBody(): String {
        return buildString {
            appendLine("### ${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}")
            appendLine()
            if (
                sharedManifestData.latestVersion != null &&
                sharedManifestData.updateState == VersionUpdateState.NewVersion
            ) {
                appendLine("#### Previous version: ${sharedManifestData.latestVersion}")
            }
        }
    }

    fun commitFiles(
        repository: GHRepository?,
        branch: GHRef?,
        files: List<Pair<String, String?>>
    ) {
        repository?.createCommit()
            ?.message(getCommitTitle())
            ?.parent(branch?.getObject()?.sha)
            ?.tree(
                repository
                    .createTree()
                    .baseTree(repository.getBranch(branch?.ref).shA1)
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
            ?.create()
            ?.also { branch?.updateTo(it.shA1) }
    }

    companion object {
        const val Microsoft = "Microsoft"
        const val wingetpkgs = "winget-pkgs"
        private const val uniqueBranchIdentifierLength = 14
    }
}
