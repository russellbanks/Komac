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
                        .state(GHIssueState.CLOSED)
                        .list()
                        .withPageSize(1)
                        .first()

                    pullRequest.let {
                        val branchName = branch.name
                        warning(
                            buildString {
                                append("Deleting ")
                                append(branchName)
                                append(" because its pull request was ")
                                append(if (it.isMerged) "merged" else "closed")
                                append(" in ")
                                append(
                                    if (it.isMerged) {
                                        "${GitHubImpl.microsoftWinGetPkgs.htmlUrl}/commit/${it.mergeCommitSha}"
                                    } else {
                                        it.htmlUrl
                                    }
                                )
                            }
                        )
                        wingetPkgsFork.getRef("heads/$branchName").delete()
                        branchesDeleted++
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
