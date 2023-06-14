package commands

import Environment
import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import github.GitHubImpl
import data.ManifestData
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import utils.versionStringComparator
import io.ExitCode
import io.Prompts
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.kohsuke.github.GitHub
import token.Token
import token.TokenStore
import github.GitHubUtils

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
        .check(Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength)) {
            it.length in minimumReasonLength..maximumReasonLength
        }

    private val submit: Boolean by option(
        "-s", "--submit",
        help = "Automatically submits a pull request with the updated pull request"
    ).flag(default = false)

    private val token: String? by option(
        "-t", "--token", "--pat", "--personal-access-token",
        help = "GitHub personal access token with the public_repo scope",
        envvar = "GITHUB_TOKEN"
    ).check("The token is invalid or has expired") { GitHub.connectUsingOAuth(it).isCredentialValid }

    override fun run(): Unit = runBlocking {
        token?.let { TokenStore.useTokenParameter(it) }
        if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
        warning("Packages should only be removed when necessary.")
        echo()
        ManifestData.packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
        if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
        ManifestData.allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, ManifestData.packageIdentifier)?.also {
            info("Found ${ManifestData.packageIdentifier} in the winget-pkgs repository")
            it.maxWithOrNull(versionStringComparator)?.let { latestVersion ->
                info("Found latest version: $latestVersion")
            }
        }
        ManifestData.packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
        GitHubImpl.microsoftWinGetPkgs.getDirectoryContent(GitHubUtils.getPackagePath(ManifestData.packageIdentifier))
            ?.find { it.name == ManifestData.packageVersion }
            ?: throw doesNotExistError(ManifestData.packageIdentifier, ManifestData.packageVersion, colors = colors)
        val deletionReason = deletionReasonParam ?: currentContext.terminal.promptForDeletionReason()
        val shouldRemoveManifest = if (submit || Environment.isCI) {
            true
        } else {
            confirm("Would you like to make a pull request to remove ${ManifestData.packageIdentifier} ${ManifestData.packageVersion}?")
                ?: throw ProgramResult(ExitCode.CtrlC)
        }
        if (!shouldRemoveManifest) return@runBlocking
        echo()
        val forkRepository = GitHubImpl.getWingetPkgsFork(currentContext.terminal)
        val ref = GitHubImpl.createBranchFromUpstreamDefaultBranch(
            winGetPkgsFork = forkRepository,
            packageIdentifier = ManifestData.packageIdentifier,
            packageVersion = ManifestData.packageVersion
        ) ?: return@runBlocking
        val directoryContent: MutableList<GHContent> = forkRepository
            .getDirectoryContent(
                GitHubUtils.getPackageVersionsPath(ManifestData.packageIdentifier, ManifestData.packageVersion),
                ref.ref
            )
        val progress = currentContext.terminal.progressAnimation {
            text("Deleting files")
            percentage()
            progressBar()
            completed()
        }
        progress.start()
        directoryContent.forEachIndexed { index, ghContent ->
            ghContent.delete("Remove: ${ghContent.name}", ref.ref)
            progress.update(index.inc().toLong(), directoryContent.size.toLong())
        }
        progress.clear()
        GitHubImpl.microsoftWinGetPkgs.createPullRequest(
            "Remove: ${ManifestData.packageIdentifier} version ${ManifestData.packageVersion}",
            "${GitHubImpl.forkOwner}:${ref.ref}",
            GitHubImpl.microsoftWinGetPkgs.defaultBranch,
            "## $deletionReason"
        ).also { success("Pull request created: ${it.htmlUrl}") }
    }

    private fun Terminal.promptForDeletionReason(): String {
        echo(colors.brightGreen("${Prompts.required} Give a reason for removing this manifest"))
        return prompt("Reason") {
            if (it.length < minimumReasonLength || it.length > maximumReasonLength) {
                ConversionResult.Invalid(Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength))
            } else {
                ConversionResult.Valid(it)
            }
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    companion object {
        const val minimumReasonLength = 4
        const val maximumReasonLength = 1000
    }
}
