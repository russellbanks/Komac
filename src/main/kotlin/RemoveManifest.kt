import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import data.GitHubImpl
import data.SharedManifestData
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.TerminalInstance

class RemoveManifest : CliktCommand(name = "remove"), KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val githubImpl by inject<GitHubImpl>()

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt()
            if (sharedManifestData.isNewPackage) {
                println(
                    brightYellow(
                        "${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository."
                    )
                )
                return@runBlocking
            }
            packageVersionPrompt()
            val forkRepository = githubImpl.getWingetPkgsFork(this)
            val ref = githubImpl.createBranch(forkRepository)
            forkRepository?.getDirectoryContent(githubImpl.baseGitHubPath, ref?.ref)?.forEach {
                it.delete("Remove: ${it.name}", ref?.ref)
            }
            githubImpl.getMicrosoftWingetPkgs()?.let { ghRepository ->
                val pullRequestTitle =
                    "Remove: ${sharedManifestData.packageIdentifier} version ${sharedManifestData.packageVersion}"
                ghRepository.createPullRequest(
                    /* title = */ pullRequestTitle,
                    /* head = */ "${githubImpl.github.myself.login}:${ref?.ref}",
                    /* base = */ ghRepository.defaultBranch,
                    /* body = */ "## This is a test for Komac"
                ).also {
                    println(it.htmlUrl)
                }
            }
        }
    }
}
