package commands

import Errors
import ExitCode
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.terminal.ConversionResult
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import input.LocaleType
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

    override fun run(): Unit = runBlocking {
        currentContext.terminal.warning(message = "Packages should only be removed when necessary.")
        echo()
        currentContext.terminal.packageIdentifierPrompt()
        if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
            throw CliktError(
                message = "${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository.",
                statusCode = 1
            )
        }
        currentContext.terminal.packageVersionPrompt()
        githubImpl.getMicrosoftWingetPkgs()?.getDirectoryContent(githubImpl.packageVersionsPath)
            ?.find { it.name == sharedManifestData.packageVersion }
            ?: throw CliktError(
                buildString {
                    append(sharedManifestData.packageIdentifier)
                    append(" ")
                    append(sharedManifestData.packageVersion)
                    append(" does not exist in ${GitHubImpl.Microsoft}/${GitHubImpl.wingetpkgs}.")
                }
            )
        val deletionReason = getDeletionReason()
        val confirmPrompt = "Would you like to make a pull request to remove " +
            "${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}?"
        val shouldRemoveManifest = confirm(confirmPrompt) ?: exitProcess(ExitCode.CtrlC.code)
        echo()
        if (shouldRemoveManifest) {
            val forkRepository = githubImpl.getWingetPkgsFork(currentContext.terminal) ?: return@runBlocking
            val ref = githubImpl.createBranchFromDefaultBranch(forkRepository, currentContext.terminal)
                ?: return@runBlocking
            val directoryContent: MutableList<GHContent> =
                forkRepository.getDirectoryContent(githubImpl.baseGitHubPath, ref.ref)
            val progress = currentContext.terminal.progressAnimation {
                text("Deleting files")
                percentage()
                progressBar()
                completed()
            }
            directoryContent.forEachIndexed { index, ghContent ->
                progress.update(index.inc().toLong(), directoryContent.size.toLong())
                ghContent.delete(/* commitMessage = */ "Remove: ${ghContent.name}", /* branch = */ ref.ref)
            }
            progress.clear()
            val ghRepository = githubImpl.getMicrosoftWingetPkgs() ?: return@runBlocking
            val pullRequestTitle =
                "Remove: ${sharedManifestData.packageIdentifier} version ${sharedManifestData.packageVersion}"
            try {
                ghRepository.createPullRequest(
                    /* title = */ pullRequestTitle,
                    /* head = */ "${githubImpl.github.await().myself.login}:${ref.ref}",
                    /* base = */ ghRepository.defaultBranch,
                    /* body = */ "## $deletionReason"
                ).also { currentContext.terminal.success("Pull request created: ${it.htmlUrl}") }
            } catch (ioException: IOException) {
                throw CliktError(message = ioException.message ?: "Failed to create pull request")
            }
        }
    }

    private fun getDeletionReason(): String {
        echo(brightGreen("${Prompts.required} Give a reason for removing this manifest"))
        return prompt(
            text = "Reason",
            convert = {
                when {
                    it.isBlank() -> ConversionResult.Invalid(Errors.blankInput(null as LocaleType?))
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

    companion object {
        const val minimumReasonLength = 4
        const val maximumReasonLength = 64
    }
}
