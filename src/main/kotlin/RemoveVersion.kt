import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextStyles.bold
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import input.PromptType
import input.Prompts
import input.Prompts.removeManifestPullRequestPrompt
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.TerminalInstance
import java.io.IOException

class RemoveVersion : CliktCommand(name = "remove"), KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val githubImpl by inject<GitHubImpl>()

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            println((bold + brightYellow)("Packages should only be removed when necessary."))
            println()
            packageIdentifierPrompt()
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
                println(
                    brightYellow(
                        "${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository."
                    )
                )
                return@runBlocking
            }
            packageVersionPrompt()
            githubImpl.getMicrosoftWingetPkgs()?.getDirectoryContent(githubImpl.packageVersionsPath)?.find {
                it.name == sharedManifestData.packageVersion
            }.let {
                if (it == null) {
                    println(
                        brightYellow(
                            buildString {
                                append(sharedManifestData.packageIdentifier)
                                append(" ")
                                append(sharedManifestData.packageVersion)
                                append(" does not exist in ${GitHubImpl.Microsoft}/${GitHubImpl.wingetpkgs}.")
                            }
                        )
                    )
                    return@runBlocking
                }
            }
            var deletionReason: String?
            do {
                println(brightGreen("${Prompts.required} Give a reason for removing this manifest"))
                deletionReason = prompt("Reason")?.trim()
                val (isReasonValid, error) = isReasonValid(deletionReason)
                error?.let { println(red(it)) }
                println()
            } while (isReasonValid != Validation.Success)
            removeManifestPullRequestPrompt(sharedManifestData = sharedManifestData).also { shouldRemoveManifest ->
                println()
                if (shouldRemoveManifest) {
                    val forkRepository = githubImpl.getWingetPkgsFork() ?: return@runBlocking
                    val ref = githubImpl.createBranchFromDefaultBranch(forkRepository) ?: return@runBlocking
                    val directoryContent: MutableList<GHContent> =
                        forkRepository.getDirectoryContent(githubImpl.baseGitHubPath, ref.ref)
                    val progress = progressAnimation {
                        text("Deleting files")
                        percentage()
                        progressBar()
                        completed()
                    }
                    directoryContent.forEachIndexed { index, ghContent ->
                        progress.update(index.inc().toLong(), directoryContent.size.toLong())
                        ghContent.delete("Remove: ${ghContent.name}", ref.ref)
                    }
                    progress.clear()
                    val ghRepository = githubImpl.getMicrosoftWingetPkgs() ?: return@runBlocking
                    val pullRequestTitle =
                        "Remove: ${sharedManifestData.packageIdentifier} version ${sharedManifestData.packageVersion}"
                    try {
                        ghRepository.createPullRequest(
                            /* title = */ pullRequestTitle,
                            /* head = */ "${githubImpl.github.myself.login}:${ref.ref}",
                            /* base = */ ghRepository.defaultBranch,
                            /* body = */ "## $deletionReason"
                        ).also { println(brightGreen("Pull request created: ${it.htmlUrl}")) }
                    } catch (ioException: IOException) {
                        println(red(ioException.message ?: "Failed to create pull request"))
                    }
                }
            }
        }
    }

    private fun isReasonValid(reason: String?): Pair<Validation, String?> {
        return when {
            reason.isNullOrBlank() -> Validation.Blank to Errors.blankInput(null as PromptType?)
            reason.length < minimumReasonLength || reason.length > maximumReasonLength -> {
                Validation.InvalidLength to Errors.invalidLength(min = minimumReasonLength, max = maximumReasonLength)
            }
            else -> Validation.Success to null
        }
    }

    companion object {
        const val minimumReasonLength = 4
        const val maximumReasonLength = 64
    }
}
