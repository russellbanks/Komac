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
}
