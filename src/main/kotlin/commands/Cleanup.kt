package commands

import com.github.ajalt.clikt.core.CliktCommand
import data.GitHubImpl

class Cleanup : CliktCommand(name = "cleanup") {
    override fun run() {
        val wingetPkgsFork = GitHubImpl.getWingetPkgsFork(currentContext.terminal)
        var branchesDeleted = 0

        wingetPkgsFork.branches.values
            .filterNot { it.name == GitHubImpl.microsoftWinGetPkgs.defaultBranch }
            .forEach { branch ->
                runCatching {
                    val commit = GitHubImpl.microsoftWinGetPkgs
                        .queryCommits()
                        .from(branch.shA1)
                        .list()
                        .withPageSize(1)
                        .first()

                    commit.let {
                        val branchName = branch.name
                        warning("Deleting $branchName because it was merged in ${it.htmlUrl}")
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
