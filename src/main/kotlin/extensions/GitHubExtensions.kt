package extensions

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.Terminal
import org.kohsuke.github.GHPullRequest
import org.kohsuke.github.GHRelease

object GitHubExtensions {
    /**
     * Extracts formatted release notes from a given release.
     *
     * 1. The function first splits the release notes body into lines and cleans each line by removing dropdowns,
     * changing all bullet points to be dashes, removing code formatted with backticks, and converting Markdown links
     * to plaintext.
     * 2. It then uses a buildString block to loop through each line of the release notes.
     * 3. If the line starts with "#" and there is another bullet point within two lines of it, it is added.
     * 4. If the line starts with "- " it is added, with each sentence being on a new line and indented.
     * 5. Finally, either the string is returned, or null if it is blank.
     *
     * @receiver release the [GHRelease] object containing the release notes to be formatted
     * @return A formatted string of the release notes or null if the release notes are blank
     */
    fun GHRelease.getFormattedReleaseNotes(): String? {
        val lines = body
            ?.replace("<details>.*</details>".toRegex(setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)), "")
            ?.lines()
            ?.map { line ->
                line.trim()
                    .let { if (it.startsWith("* ")) it.replaceFirst("* ", "- ") else it }
                    .replace("~+([^~]+)~+".toRegex(), "$1")
                    .replace("\\*+([^*]+)\\*+".toRegex(), "$1")
                    .replace("`", "")
                    .replace("\\[?!\\[(.*?)]\\((.*?)\\)(?:]\\((.*?)\\))?".toRegex(), "")
                    .replace("\\[([^]]+)]\\([^)]+\\)".toRegex(), "$1")
                    .replace("[a-fA-F0-9]{40}".toRegex(), "")
                    .replace("https?://github.com/([\\w-]+)/([\\w-]+)/(pull|issues)/(\\d+)".toRegex()) {
                        val urlRepository = "${it.groupValues[1]}/${it.groupValues[2]}"
                        val issueNumber = it.groupValues[4]
                        if (urlRepository == owner.fullName) "#$issueNumber" else "$urlRepository#$issueNumber"
                    }
                    .trim()
            }
        return buildString {
            lines?.forEachIndexed { index, line ->
                when {
                    line.startsWith("#") && (1..2).any { lines.getOrNull(index + it)?.startsWith("- ") == true } -> {
                        appendLine(line.dropWhile { it == '#' }.trimEnd())
                    }
                    line.startsWith("- ") -> {
                        appendLine(
                            "- ${line.replace("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))".toRegex(), "$1\n ").drop(2).trim()}"
                        )
                    }
                }
            }
        }.trim().takeUnless(String::isBlank)
    }

    /**
     * Prints the result of a GitHub pull request creation to the provided [terminal].
     *
     * @param terminal The [Terminal] instance to use for printing the result
     * @throws CliktError if the pull request creation failed and the receiver [GHPullRequest] is null
     */
    infix fun GHPullRequest?.printResultTo(terminal: Terminal) {
        if (this != null) {
            terminal.success("Pull request created: $htmlUrl")
        } else {
            throw CliktError("Failed to create pull request", statusCode = 1)
        }
    }
}
