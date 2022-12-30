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
import schemas.ManifestBuilder
import java.io.IOException
import java.nio.file.Path

@Single
class GitHubImpl : KoinComponent {
    val github: GitHub = GitHubBuilder().withOAuthToken(token).build()
    private val sharedManifestData: SharedManifestData by inject()
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

    fun commitFiles(repository: GHRepository?, branch: GHRef?, versionDirectory: Path) {
        repository?.createCommit()
            ?.message("This is a test commit for Komac")
            ?.parent(branch?.getObject()?.sha)
            ?.tree(
                repository
                    .createTree()
                    .baseTree(repository.getBranch(branch?.ref).shA1)
                    .add(
                        ManifestBuilder.installerManifestGitHubPath,
                        versionDirectory.resolve(ManifestBuilder.installerManifestName).toFile().readBytes(),
                        false
                    )
                    .add(
                        ManifestBuilder.defaultLocaleManifestGitHubPath,
                        versionDirectory.resolve(ManifestBuilder.defaultLocaleManifestName).toFile().readBytes(),
                        false
                    )
                    .add(
                        ManifestBuilder.versionManifestGitHubPath,
                        versionDirectory.resolve(ManifestBuilder.versionManifestName).toFile().readBytes(),
                        false
                    )
                    .apply {
                        val regex = Regex("${Regex.escape(sharedManifestData.packageIdentifier)}.locale\\.(.*)\\.yaml")
                        versionDirectory.toFile().listFiles { _, name ->
                            name.matches(regex) && !name.contains(sharedManifestData.defaultLocale)
                        }?.forEach { file ->
                            regex.find(file.name)?.groupValues?.get(1)?.let {
                                println(it)
                                add(
                                    ManifestBuilder.getLocaleManifestGitHubPath(it),
                                    file.readBytes(),
                                    false
                                )
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
        private const val token = "TOKEN"
        const val Microsoft = "Microsoft"
        const val wingetpkgs = "winget-pkgs"
        private const val uniqueBranchIdentifierLength = 14
    }
}
