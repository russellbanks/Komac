package commands

import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.validate
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.GitHubImpl
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import extensions.versionStringComparator
import input.ExitCode
import input.Prompts
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.kohsuke.github.GitHub
import token.Token
import token.TokenStore
import utils.GitHubUtils

class RemoveVersion : CliktCommand(name = "remove") {
    private val packageIdentifierParam: String? by option("--id", "--package-identifier")
    private val packageVersionParam: String? by option("--version", "--package-version")
    private val deletionReasonParam: String? by option("--reason", "--reason-for-deletion", "--deletion-reason")
        .validate {
            require(it.length in minimumReasonLength..maximumReasonLength) {
                colors.danger(Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength))
            }
        }
    private val submit: Boolean by option().flag(default = false)
    private val token: String? by option("-t", "--token", envvar = "GITHUB_TOKEN").validate {
        require(GitHub.connectUsingOAuth(it).isCredentialValid) {
            colors.danger("The token is invalid or has expired")
        }
    }

    override fun run(): Unit = runBlocking {
        val terminal = currentContext.terminal
        token?.let { TokenStore.useTokenParameter(it) }
        with(AllManifestData) {
            if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
            warning("Packages should only be removed when necessary.")
            echo()
            packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
            if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(terminal)
            allVersions = GitHubUtils.getAllVersions(GitHubImpl.microsoftWinGetPkgs, packageIdentifier)
            info("Found $packageIdentifier in the winget-pkgs repository")
            allVersions?.maxWithOrNull(versionStringComparator)?.let { latestVersion ->
                info("Found latest version: $latestVersion")
            }
            packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
            GitHubImpl.microsoftWinGetPkgs.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
                ?.find { it.name == packageVersion }
                ?: throw doesNotExistError(packageIdentifier, packageVersion, colors = colors)
            val deletionReason = deletionReasonParam ?: terminal.promptForDeletionReason()
            val shouldRemoveManifest = if (submit || System.getenv("CI")?.toBooleanStrictOrNull() == true) {
                true
            } else {
                confirm("Would you like to make a pull request to remove $packageIdentifier $packageVersion?")
                    ?: throw ProgramResult(ExitCode.CtrlC)
            }
            if (!shouldRemoveManifest) return@runBlocking
            echo()
            val forkRepository = GitHubImpl.getWingetPkgsFork(terminal)
            val ref = GitHubImpl.createBranchFromUpstreamDefaultBranch(
                winGetPkgsFork = forkRepository,
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion
            ) ?: return@runBlocking
            val directoryContent: MutableList<GHContent> = forkRepository
                .getDirectoryContent(GitHubUtils.getPackageVersionsPath(packageIdentifier, packageVersion), ref.ref)
            val progress = terminal.progressAnimation {
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
                "Remove: $packageIdentifier version $packageVersion",
                "${GitHubImpl.forkOwner}:${ref.ref}",
                GitHubImpl.microsoftWinGetPkgs.defaultBranch,
                "## $deletionReason"
            ).also { success("Pull request created: ${it.htmlUrl}") }
        }
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
        const val maximumReasonLength = 128
    }
}
