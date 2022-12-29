package data

import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single

@Single
class GitHubImpl {
    val github: GitHub = GitHubBuilder().withOAuthToken(token).build()

    companion object {
        private const val token = "TOKEN"
    }
}
