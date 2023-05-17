package commands

import Environment
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
import data.GitHubImpl
import data.ManifestData
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

class RemoveVersion : CliktCommand(
    help = """
        Removes a pre-existing package version in winget-pkgs
        
        To remove an entire package, all versions of that package must be removed
    """.trimIndent(),
    name = "remove"
) {
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
        val shouldRemoveManifest = if (submit || Environment.isCI) true else {
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
            .getDirectoryContent(GitHubUtils.getPackageVersionsPath(
                ManifestData.packageIdentifier,
                ManifestData.packageVersion
            ), ref.ref)
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
