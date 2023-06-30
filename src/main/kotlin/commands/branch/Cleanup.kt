package commands.branch

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.check
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import commands.info
import commands.prompt
import commands.success
import commands.warning
import github.GitHubImpl
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHIssueState
import org.kohsuke.github.GitHub
import token.Token
import token.TokenStore

class Cleanup : CliktCommand(
    help = "Deletes branches that have had a merged or closed pull request associated with them",
    name = "cleanup"
) {
    private val onlyMerged: Boolean by option(help = "Only delete merged branches").flag(default = false)
    private val onlyClosed: Boolean by option(help = "Only delete closed branches").flag(default = false)
    private val tokenParameter: String? by option(
        "-t", "--token", "--pat", "--personal-access-token",
        help = "GitHub personal access token with the public_repo scope",
        envvar = "GITHUB_TOKEN"
    ).check("The token is invalid or has expired") { GitHub.connectUsingOAuth(it).isCredentialValid }

    override fun run() = runBlocking {
        tokenParameter?.let { TokenStore.useTokenParameter(it) }
        if (TokenStore.token == null) prompt(Token).also { TokenStore.putToken(it) }
        if (!TokenStore.isTokenValid.await()) TokenStore.invalidTokenPrompt(currentContext.terminal)
        val mergeState = when {
            onlyMerged && !onlyClosed -> "merged"
            !onlyMerged && onlyClosed -> "closed"
            else -> "merged or closed"
        }
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
                    if ((onlyMerged && it.isMerged) || (onlyClosed && !it.isMerged) || (!onlyMerged && !onlyClosed)) {
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
