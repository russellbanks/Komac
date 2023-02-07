package commands

import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandUtils.prompt
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import data.shared.PackageIdentifier
import data.shared.PackageIdentifier.getLatestVersion
import data.shared.PackageVersion
import input.ExitCode
import input.Prompts
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.io.IOException
import kotlin.system.exitProcess

class RemoveVersion : CliktCommand(name = "remove"), KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val githubImpl by inject<GitHubImpl>()
    private val identifierParam: String? by option("--id", "--package-identifier")
    var packageIdentifier: String? = null
    var packageVersion: String? = null

    override fun run(): Unit = runBlocking {
        with (currentContext.terminal) {
            warning(message = "Packages should only be removed when necessary.")
            echo()
            packageIdentifier = prompt(PackageIdentifier, parameter = identifierParam)
            sharedManifestData.latestVersion = getLatestVersion(sharedManifestData.packageIdentifier)
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) { throw doesNotExistError() }
            packageVersion = prompt(PackageVersion)
            githubImpl.getMicrosoftWingetPkgs()?.getDirectoryContent(githubImpl.packageVersionsPath)
                ?.find { it.name == sharedManifestData.packageVersion }
                ?: throw doesNotExistError()
            val deletionReason = promptForDeletionReason()
            val shouldRemoveManifest = confirm(
                text = "Would you like to make a pull request to remove $packageIdentifier $packageVersion?"
            ) ?: exitProcess(ExitCode.CtrlC.code)
            if (!shouldRemoveManifest) return@runBlocking
            echo()
            val forkRepository = githubImpl.getWingetPkgsFork(this) ?: return@runBlocking
            val ref = githubImpl.createBranchFromDefaultBranch(forkRepository, this) ?: return@runBlocking
            val directoryContent: MutableList<GHContent> =
                forkRepository.getDirectoryContent(githubImpl.baseGitHubPath, ref.ref)
            val progress = progressAnimation {
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
            val wingetPkgs = githubImpl.getMicrosoftWingetPkgs() ?: return@runBlocking
            try {
                wingetPkgs.createPullRequest(
                    /* title = */ "Remove: $packageIdentifier version $packageVersion",
                    /* head = */ "${githubImpl.github.await().myself.login}:${ref.ref}",
                    /* base = */ wingetPkgs.defaultBranch,
                    /* body = */ "## $deletionReason"
                ).also { success("Pull request created: ${it.htmlUrl}") }
            } catch (ioException: IOException) {
                throw CliktError(message = ioException.cause?.message ?: "Failed to create pull request")
            }
        }
    }

    private fun Terminal.promptForDeletionReason(): String {
        echo(colors.brightGreen("${Prompts.required} Give a reason for removing this manifest"))
        return prompt(
            text = "Reason",
            convert = {
                when {
                    it.isBlank() -> ConversionResult.Invalid(Errors.blankInput(null as String?))
                    it.length < minimumReasonLength || it.length > maximumReasonLength -> {
                        ConversionResult.Invalid(
                            Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength)
                        )
                    }
                    else -> ConversionResult.Valid(it)
                }
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
    }

    private fun doesNotExistError() = CliktError(
        message = buildString {
            append(sharedManifestData.packageIdentifier)
            append(" ")
            append(sharedManifestData.packageVersion)
            append(" does not exist in ${GitHubImpl.Microsoft}/${GitHubImpl.wingetpkgs}.")
        },
        statusCode = 1
    )

    companion object {
        const val minimumReasonLength = 4
        const val maximumReasonLength = 64
    }
}
