package input

import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal

object Prompts {
    const val required = "[Required]"
    const val optional = "[Optional]"

    const val optionIndent = 3

    const val enterChoice = "Enter Choice"

    fun Terminal.pullRequestPrompt(packageIdentifier: String, packageVersion: String): ManifestResultOption? {
        println(
            verticalLayout {
                cell(colors.info("What would you like to do with $packageIdentifier $packageVersion?"))
                ManifestResultOption.values().forEach {
                    cell(
                        colors.brightWhite(
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
            prompt = enterChoice,
            default = ManifestResultOption.Quit,
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
