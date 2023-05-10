package commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import data.GitHubImpl
import org.kohsuke.github.GHIssueState

class Cleanup : CliktCommand(name = "cleanup") {
    private val onlyMerged: Boolean by option().flag(default = false)

    override fun run() {
        val mergeState = if (onlyMerged) "merged" else "merged or closed"
        info("Deleting branches with a $mergeState pull request to ${GitHubImpl.wingetPkgsFullName} from them")
        val wingetPkgsFork = GitHubImpl.getWingetPkgsFork(currentContext.terminal)
        var branchesDeleted = 0

        wingetPkgsFork.branches.values
            .filterNot { it.name == GitHubImpl.microsoftWinGetPkgs.defaultBranch }
            .forEach { branch ->
                runCatching {
                    val pullRequest = GitHubImpl.microsoftWinGetPkgs
                        .queryPullRequests()
                        .head("${wingetPkgsFork.ownerName}:${branch.name}")
                        .base(GitHubImpl.microsoftWinGetPkgs.defaultBranch)
                        .state(GHIssueState.CLOSED)
                        .list()
                        .withPageSize(1)
                        .first()

                    pullRequest?.let {
                        if (!onlyMerged || it.isMerged) {
                            val branchName = branch.name
                            val action = if (it.isMerged) "merged" else "closed"
                            val url = if (it.isMerged) {
                                "${GitHubImpl.microsoftWinGetPkgs.htmlUrl}/commit/${it.mergeCommitSha}"
                            } else {
                                it.htmlUrl
                            }
                            warning("Deleting $branchName because its pull request was $action in $url")
                            wingetPkgsFork.getRef("heads/$branchName").delete()
                            branchesDeleted++
                        }
                    }
                }
            }

        if (branchesDeleted > 0) {
            success("$branchesDeleted branches were deleted")
        } else {
            echo("No branches were found that could be deleted")
        }
    }
}
