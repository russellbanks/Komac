package data

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
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
import schemas.TerminalInstance
import token.TokenStore
import java.io.IOException

@Single
class GitHubImpl : KoinComponent {
    val github: GitHub = GitHubBuilder()
        .withConnector(KtorGitHubConnector(get<Clients>().httpClient))
        .withOAuthToken(get<TokenStore>().token)
        .build()
    private val sharedManifestData: SharedManifestData by inject()
    val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    val defaultLocaleManifestName
        get() = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    val versionManifestName = "${sharedManifestData.packageIdentifier}.yaml"
    private val terminal = get<TerminalInstance>().terminal

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

    fun getWingetPkgsFork(): GHRepository? {
        with(terminal) {
            return try {
                github.getRepository("${github.myself.login}/$wingetpkgs").also {
                    println(brightWhite("Found forked $wingetpkgs repository: ${it.fullName}"))
                }
            } catch (_: IOException) {
                println(brightWhite("Fork of $wingetpkgs not found. Forking..."))
                try {
                    github.getRepository("$Microsoft/$wingetpkgs").fork().also {
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

    fun getMicrosoftWingetPkgs(): GHRepository? {
        return try {
            github.getRepository("$Microsoft/$wingetpkgs")
        } catch (ioException: IOException) {
            terminal.println(brightRed(ioException.message ?: "Failed to get Microsoft winget-pkgs repository."))
            null
        }
    }

    fun createBranchFromDefaultBranch(repository: GHRepository): GHRef? {
        return try {
            repository.createRef(
                /* name = */ "refs/heads/$branchName",
                /* sha = */ repository.getBranch(repository.defaultBranch).shA1
            )
        } catch (ioException: IOException) {
            terminal.println(brightRed(ioException.message ?: "Failed to create branch."))
            null
        }
    }

    private fun getCommitMessage() = buildString {
        append(sharedManifestData.updateState)
        append(": ")
        append(sharedManifestData.packageIdentifier)
        append(" ")
        append(sharedManifestData.packageVersion)
    }

    fun commitFiles(
        repository: GHRepository?,
        branch: GHRef?,
        files: List<Pair<String, String?>>
    ) {
        repository?.createCommit()
            ?.message(getCommitMessage())
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
