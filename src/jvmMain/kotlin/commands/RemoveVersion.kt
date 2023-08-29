package commands

import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.core.terminal
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.VersionUpdateState
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import github.GitHubImpl
import github.GitHubUtils
import io.ExitCode
import io.Prompts
import io.menu.yesNoMenu
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.kohsuke.github.GitHub
import utils.Environment
import utils.versionStringComparator

class RemoveVersion : CliktCommand(
    help = """
        Removes a pre-existing package version in winget-pkgs
        
        To remove an entire package, all versions of that package must be removed
    """.trimIndent(),
    name = "remove"
) {
    private val packageIdentifierParam: String? by option(
        "-i", "--id", "--identifier", "--package-identifier",
        help = "Package identifier. Example: Publisher.Package"
    )

    private val packageVersionParam: String? by option(
        "-v", "--version", "--package-version",
        help = "Package version. Example: 1.2.3"
    )

    private val deletionReasonParam: String? by option("--reason", "--reason-for-deletion", "--deletion-reason")
        .check(Errors.invalidLength(min = REASON_MIN_LENGTH, max = REASON_MAX_LENGTH)) {
            it.length in REASON_MIN_LENGTH..REASON_MAX_LENGTH
        }

    private val submit: Boolean by option(
        "-s", "--submit",
        help = "Automatically submits a pull request with the updated pull request"
    ).flag(default = false)

    private val tokenParameter: String? by option(
        "-t", "--token", "--pat", "--personal-access-token",
        help = "GitHub personal access token with the public_repo scope",
        envvar = "GITHUB_TOKEN"
    ).check("The token is invalid or has expired") { GitHub.connectUsingOAuth(it).isCredentialValid }

    private lateinit var packageIdentifier: String
    private lateinit var packageVersion: String

    override fun run(): Unit = runBlocking {
        handleToken(tokenParameter)
        warning("Packages should only be removed when necessary.")
        echo()
        packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
        GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)?.also { allVersions ->
            info("Found $packageIdentifier in the winget-pkgs repository")
            allVersions.maxWithOrNull(versionStringComparator)?.let { latestVersion ->
                info("Found latest version: $latestVersion")
            }
        }
        packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
        GitHubImpl.microsoftWinGetPkgs.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
            ?.find { it.name == packageVersion }
            ?: throw doesNotExistError(packageIdentifier, packageVersion, theme = theme)
        val deletionReason = deletionReasonParam ?: terminal.promptForDeletionReason()
        val shouldRemoveManifest = if (submit || Environment.isCI) {
            true
        } else {
            info("Would you like to make a pull request to remove $packageIdentifier $packageVersion?")
            terminal.yesNoMenu(default = true).prompt()
        }
        if (!shouldRemoveManifest) return@runBlocking
        echo()
        val forkRepository = GitHubImpl.getWingetPkgsFork(terminal)
        val pullRequestBranch = GitHubImpl.createBranchFromUpstreamDefaultBranch(
            winGetPkgsFork = forkRepository,
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion
        )
        val directoryContent: List<GHContent> = forkRepository
            .getDirectoryContent(GitHubUtils.getPackageVersionsPath(packageIdentifier, packageVersion), pullRequestBranch.ref)
        val title = GitHubUtils.getCommitTitle(packageIdentifier, packageVersion, VersionUpdateState.RemoveVersion)
        forkRepository.createCommit()
            ?.message(title)
            ?.parent(pullRequestBranch.getObject()?.sha)
            ?.tree(
                forkRepository.createTree()
                    .baseTree(forkRepository.getBranch(pullRequestBranch.ref).shA1)
                    .apply {
                        for (file in directoryContent) {
                            delete(file.path)
                        }
                    }
                    .create()
                    .sha
            )
            ?.create()
            ?.also { pullRequestBranch.updateTo(it.shA1) }
        GitHubImpl.microsoftWinGetPkgs.createPullRequest(
            title,
            "${GitHubImpl.forkOwner}:${pullRequestBranch.ref}",
            GitHubImpl.microsoftWinGetPkgs.defaultBranch,
            "## $deletionReason"
        ).also { success("Pull request created: ${it.htmlUrl}") }
    }

    private fun Terminal.promptForDeletionReason(): String {
        echo(TextColors.brightGreen("${Prompts.REQUIRED} Give a reason for removing this manifest"))
        return prompt("Reason") {
            if (it.length < REASON_MIN_LENGTH || it.length > REASON_MAX_LENGTH) {
                ConversionResult.Invalid(Errors.invalidLength(min = REASON_MIN_LENGTH, max = REASON_MAX_LENGTH))
            } else {
                ConversionResult.Valid(it)
            }
        } ?: throw ProgramResult(ExitCode.CTRLC)
    }

    companion object {
        const val REASON_MIN_LENGTH = 4
        const val REASON_MAX_LENGTH = 1000
    }
}
