import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextStyles.bold
import com.github.ajalt.mordant.table.verticalLayout
import data.GitHubImpl
import data.SharedManifestData
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import input.Polar
import input.PromptType
import input.Prompts
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
            println((bold + brightYellow)("Packages should only be removed when necessary."))
            println()
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
            println(
                verticalLayout {
                    cell(
                        brightYellow(
                            "Would you like to make a pull request to remove " +
                                "${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}?"
                        )
                    )
                    Polar.values().forEach {
                        cell(brightWhite("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                    }
                }
            )
            val input: Char? = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                showChoices = false,
                choices = Polar.values().map { it.name.first().toString() },
            )?.trim()?.firstOrNull()
            println()
            if (input == Polar.Yes.toString().first()) {
                val forkRepository = githubImpl.getWingetPkgsFork(this)
                val ref = githubImpl.createBranch(forkRepository)
                val directoryContent = forkRepository?.getDirectoryContent(githubImpl.baseGitHubPath, ref?.ref)
                val progress = progressAnimation {
                    text("Deleting files")
                    percentage()
                    progressBar()
                    completed()
                }
                directoryContent?.forEachIndexed { index, ghContent ->
                    progress.update(index.inc().toLong(), directoryContent.size.toLong())
                    ghContent.delete("Remove: ${ghContent.name}", ref?.ref)
                }
                progress.clear()
                githubImpl.getMicrosoftWingetPkgs()?.let { ghRepository ->
                    val pullRequestTitle =
                        "Remove: ${sharedManifestData.packageIdentifier} version ${sharedManifestData.packageVersion}"
                    ghRepository.createPullRequest(
                        /* title = */ pullRequestTitle,
                        /* head = */ "${githubImpl.github.myself.login}:${ref?.ref}",
                        /* base = */ ghRepository.defaultBranch,
                        /* body = */ "## $deletionReason"
                    ).also {
                        println(it.htmlUrl)
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
