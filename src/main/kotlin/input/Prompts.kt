package input

import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.SharedManifestData

object Prompts {
    const val required = "[Required]"
    const val optional = "[Optional]"

    const val optionIndent = 3

    const val enterChoice = "Enter Choice"

    const val noIdea = "No idea"

    fun Terminal.pullRequestPrompt(sharedManifestData: SharedManifestData): ManifestResultOption? {
        println(
            verticalLayout {
                cell(
                    brightYellow(
                        "What would you like to do with " +
                            "${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}?"
                    )
                )
                ManifestResultOption.values().forEach {
                    cell(
                        brightWhite(
                            buildString {
                                append(" ".repeat(optionIndent))
                                append("[${it.toString().first().titlecase()}] ")
                                append(it.toString().replaceFirstChar { it.titlecase() })
                            }
                        )
                    )
                }
            }
        )
        return prompt(
            prompt = brightWhite(enterChoice),
            convert = {
                ConversionResult.Valid(
                    when (it.firstOrNull()?.lowercase()) {
                        ManifestResultOption.PullRequest.name.first().lowercase() -> ManifestResultOption.PullRequest
                        ManifestResultOption.WriteToFiles.name.first().lowercase() -> ManifestResultOption.WriteToFiles
                        else -> ManifestResultOption.Quit
                    }
                )
            }
        )
    }

    fun Terminal.removeManifestPullRequestPrompt(sharedManifestData: SharedManifestData): Boolean {
        println(
            verticalLayout {
                cell(
                    brightYellow(
                        "Would you like to make a pull request to remove " +
                            "${sharedManifestData.packageIdentifier} ${sharedManifestData.packageVersion}?"
                    )
                )
                Polar.values().forEach {
                    cell(brightWhite("${" ".repeat(optionIndent)} [${it.name.first()}] ${it.name}"))
                }
            }
        )
        return prompt(
            prompt = brightWhite(enterChoice),
            convert = {
                when (it.firstOrNull()?.lowercase()) {
                    Polar.Yes.name.first().lowercase() -> ConversionResult.Valid(Polar.Yes)
                    Polar.No.name.first().lowercase() -> ConversionResult.Valid(Polar.No)
                    else -> ConversionResult.Invalid("Invalid choice")
                }
            }
        ).let { it == Polar.Yes }
    }
}
