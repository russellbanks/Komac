package commands

import com.github.ajalt.clikt.core.CliktCommand
import data.GitHubImpl
import org.kohsuke.github.GHIssueState

class Cleanup : CliktCommand(name = "cleanup") {
    override fun run() {
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
                        .state(GHIssueState.ALL)
                        .list()
                        .withPageSize(1)
                        .first()

                    pullRequest.let {
                        if (it.isMerged) {
                            val branchName = branch.name
                            warning("Deleting $branchName because it was merged in ${GitHubImpl.microsoftWinGetPkgs.htmlUrl}/commit/${it.mergeCommitSha}")
                            wingetPkgsFork.getRef("heads/$branchName").delete()
                            branchesDeleted++
                        }
                    }
                }
            }

        if (branchesDeleted == 0) {
            echo("No branches were found that could be deleted")
        } else {
            success("$branchesDeleted branches were deleted")
        }
    }
}
