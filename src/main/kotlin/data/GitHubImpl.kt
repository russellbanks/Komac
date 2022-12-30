package data

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import java.io.IOException

@Single
class GitHubImpl : KoinComponent {
    val github: GitHub = GitHubBuilder().withOAuthToken(token).build()
    private val sharedManifestData: SharedManifestData by inject()
    private val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    private val defaultLocaleManifestName
        get() = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    private val versionManifestName = "${sharedManifestData.packageIdentifier}.version.yaml"

    val baseGitHubPath
        get() = buildString {
            append("manifests/")
            append("${sharedManifestData.packageIdentifier.first().lowercase()}/")
            append("${sharedManifestData.packageIdentifier.replace(".", "/")}/")
            append(sharedManifestData.packageVersion)
        }

    val installerManifestGitHubPath
        get() = "$baseGitHubPath/$installerManifestName"

    val defaultLocaleManifestGitHubPath
        get() = "$baseGitHubPath/$defaultLocaleManifestName"

    val versionManifestGitHubPath
        get() = "$baseGitHubPath/$versionManifestName"

    fun getLocaleManifestGitHubPath(locale: String): String {
        return "$baseGitHubPath/${sharedManifestData.packageIdentifier}.locale.$locale.yaml"
    }

    private val branchName: String
        get() = buildString {
            append(sharedManifestData.packageIdentifier)
            append("-")
            append(sharedManifestData.packageVersion)
            append("-")
            append(List(uniqueBranchIdentifierLength) { (('A'..'Z') + ('0'..'9')).random() }.joinToString(""))
        }

    fun getWingetPkgsFork(terminal: Terminal): GHRepository? {
        with(terminal) {
            return try {
                github.getRepository("${github.myself.login}/$wingetpkgs").also {
                    println(brightWhite("Found forked winget-pkgs repository: ${it.fullName}"))
                }
            } catch (_: IOException) {
                println(brightWhite("Fork of winget-pkgs not found. Forking..."))
                try {
                    github.getRepository("$Microsoft/$wingetpkgs").fork().also {
                        println(brightGreen("Forked winget-pkgs repository: ${it.fullName}"))
                    }
                } catch (_: IOException) {
                    println(red("Failed to fork winget-pkgs. Please try again or fork it manually."))
                    null
                }
            }
        }
    }

    fun getMicrosoftWingetPkgs(): GHRepository? {
        return try {
            github.getRepository("$Microsoft/$wingetpkgs")
        } catch (_: IOException) {
            null
        }
    }

    fun createBranch(repository: GHRepository?): GHRef? {
        return repository?.createRef(
            /* name = */ "refs/heads/$branchName",
            /* sha = */ repository.getBranch(repository.defaultBranch).shA1
        )
    }

    fun commitFiles(
        repository: GHRepository?,
        branch: GHRef?,
        files: List<Pair<String, String?>>
    ) {
        repository?.createCommit()
            ?.message("This is a test commit for Komac")
            ?.parent(branch?.getObject()?.sha)
            ?.tree(
                repository
                    .createTree()
                    .baseTree(repository.getBranch(branch?.ref).shA1)
                    .apply {
                        files.forEach { (path, content) ->
                            if (content != null) {
                                add(path, content, false)
                            }
                        }
                    }
                    .create()
                    .sha
            )
            ?.create()
            ?.also { branch?.updateTo(it.shA1) }
    }

    fun buildManifestString(schemaUrl: String, block: StringBuilder.() -> Unit): String {
        return buildString {
            appendLine(Schemas.Comments.createdBy)
            appendLine(Schemas.Comments.languageServer(schemaUrl))
            appendLine()
            block()
        }
    }

    companion object {
        private const val token = "TOKEN"
        const val Microsoft = "Microsoft"
        const val wingetpkgs = "winget-pkgs"
        private const val uniqueBranchIdentifierLength = 14
    }
}
