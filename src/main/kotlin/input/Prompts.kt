package input

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal

object Prompts {
    const val required = "[Required]"
    const val optional = "[Optional]"

    const val optionIndent = 3

    const val enterChoice = "Enter Choice"

    @OptIn(ExperimentalStdlibApi::class)
    fun Terminal.pullRequestPrompt(packageIdentifier: String, packageVersion: String): ManifestResultOption {
        println(
            verticalLayout {
                cell(colors.info("What would you like to do with $packageIdentifier $packageVersion?"))
                for (option in ManifestResultOption.entries) {
                    cell(
                        colors.brightWhite(
                            buildString {
                                append(" ".repeat(optionIndent))
                                append("[${option.toString().first().titlecase()}] ")
                                append(option.toString().replaceFirstChar(Char::titlecase))
                            }
                        )
                    )
                }
            }
        )
        return prompt(prompt = enterChoice, default = ManifestResultOption.Quit) {
            ConversionResult.Valid(
                when (it.firstOrNull()?.lowercase()) {
                    ManifestResultOption.PullRequest.name.first().lowercase() -> ManifestResultOption.PullRequest
                    ManifestResultOption.WriteToFiles.name.first().lowercase() -> ManifestResultOption.WriteToFiles
                    else -> ManifestResultOption.Quit
                }
            )
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }
}
