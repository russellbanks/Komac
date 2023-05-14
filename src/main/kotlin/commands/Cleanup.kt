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

        val branches = wingetPkgsFork.branches.values.filterNot {
            it.name == GitHubImpl.microsoftWinGetPkgs.defaultBranch
        }

        for (branch in branches) {
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
                        warning("Deleting $branchName because ${it.htmlUrl} was $action")
                        wingetPkgsFork.getRef("heads/$branchName").delete()
                        branchesDeleted++
                    }
                }
            }
        }

        if (branchesDeleted > 0) {
            success("$branchesDeleted out of ${branches.size} total branches were deleted")
        } else {
            info("No branches were found that could be deleted")
        }
    }
}
