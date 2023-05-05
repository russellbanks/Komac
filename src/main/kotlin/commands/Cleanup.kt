package commands

import com.github.ajalt.clikt.core.CliktCommand
import data.GitHubImpl

class Cleanup : CliktCommand(name = "cleanup") {
    override fun run() {
        val fork = GitHubImpl.getWingetPkgsFork(currentContext.terminal)
        fork.branches.values
            .filterNot { it.name == GitHubImpl.microsoftWinGetPkgs.defaultBranch }
            .forEach { branch ->
                runCatching {
                    GitHubImpl.microsoftWinGetPkgs.queryCommits().from(branch.shA1).list().withPageSize(1).first().let {
                        warning("Deleting ${branch.name} because it was merged in ${it.htmlUrl}")
                        fork.getRef("heads/${branch.name}").delete()
                    }
                }
            }
    }
}
