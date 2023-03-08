package commands

import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.GitHubImpl
import data.VersionUpdateState
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import data.shared.PackageVersion.getHighestVersion
import input.ExitCode
import input.Prompts
import kotlinx.coroutines.runBlocking
import network.Http
import org.kohsuke.github.GHContent
import token.Token
import token.TokenStore
import utils.GitHubUtils

class RemoveVersion : CliktCommand(name = "remove") {
    private val allManifestData = AllManifestData()
    private val tokenStore = TokenStore()
    private val client = Http.client
    private val gitHubImpl by lazy { GitHubImpl(tokenStore.token as String, client) }
    private val identifierParam: String? by option("--id", "--package-identifier")

    override fun run(): Unit = runBlocking {
        with(allManifestData) {
            if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
            warning(message = "Packages should only be removed when necessary.")
            echo()
            packageIdentifier = prompt(PackageIdentifier, parameter = identifierParam)
            if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(gitHubImpl.getMicrosoftWinGetPkgs(), packageIdentifier)
            latestVersion = allVersions?.getHighestVersion()?.also {
                if (System.getenv("CI")?.toBooleanStrictOrNull() != true) {
                    info("Found $packageIdentifier in the winget-pkgs repository")
                    info("Found latest version: $it")
                }
            }
            if (updateState == VersionUpdateState.NewPackage) {
                throw doesNotExistError(packageIdentifier, packageVersion)
            }
            packageVersion = prompt(PackageVersion)
            gitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            gitHubImpl.getMicrosoftWinGetPkgs()?.getDirectoryContent(GitHubUtils.getPackagePath(packageIdentifier))
                ?.find { it.name == packageVersion }
                ?: throw doesNotExistError(packageIdentifier, packageVersion)
            val deletionReason = currentContext.terminal.promptForDeletionReason()
            val shouldRemoveManifest = confirm(
                text = "Would you like to make a pull request to remove $packageIdentifier $packageVersion?"
            ) ?: throw ProgramResult(ExitCode.CtrlC.code)
            if (!shouldRemoveManifest) return@runBlocking
            echo()
            val forkRepository = gitHubImpl.getWingetPkgsFork(currentContext.terminal) ?: return@runBlocking
            val ref = gitHubImpl.createBranchFromUpstreamDefaultBranch(
                repository = forkRepository,
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                terminal = currentContext.terminal
            ) ?: return@runBlocking
            val directoryContent: MutableList<GHContent> = forkRepository
                .getDirectoryContent(GitHubUtils.getPackageVersionsPath(packageIdentifier, packageVersion), ref.ref)
            val progress = currentContext.terminal.progressAnimation {
                text("Deleting files")
                percentage()
                progressBar()
                completed()
            }
            progress.start()
            directoryContent.forEachIndexed { index, ghContent ->
                ghContent.delete(/* commitMessage = */ "Remove: ${ghContent.name}", /* branch = */ ref.ref)
                progress.update(index.inc().toLong(), directoryContent.size.toLong())
            }
            progress.clear()
            val wingetPkgs = gitHubImpl.getMicrosoftWinGetPkgs() ?: return@runBlocking
            wingetPkgs.createPullRequest(
                /* title = */ "Remove: $packageIdentifier version $packageVersion",
                /* head = */ "${gitHubImpl.forkOwner}:${ref.ref}",
                /* base = */ wingetPkgs.defaultBranch,
                /* body = */ "## $deletionReason"
            ).also { success("Pull request created: ${it.htmlUrl}") }
        }
    }

    private fun Terminal.promptForDeletionReason(): String {
        echo(colors.brightGreen("${Prompts.required} Give a reason for removing this manifest"))
        return prompt("Reason") {
            when {
                it.isBlank() -> ConversionResult.Invalid(Errors.blankInput(null as String?))
                it.length < minimumReasonLength || it.length > maximumReasonLength -> {
                    ConversionResult.Invalid(
                        Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength)
                    )
                }
                else -> ConversionResult.Valid(it)
            }
        } ?: throw ProgramResult(ExitCode.CtrlC.code)
    }

    companion object {
        const val minimumReasonLength = 4
        const val maximumReasonLength = 128
    }
}
