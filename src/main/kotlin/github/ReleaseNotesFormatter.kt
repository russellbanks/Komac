package github

import org.kohsuke.github.GHRelease
import utils.EmojisUnicodeMapper

object ReleaseNotesFormatter {
    const val maxCharacterLimit = 10_000

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
    val GHRelease.formattedReleaseNotes: String?
        get() {
            val lines = body
                ?.replace("<details>.*</details>".toRegex(setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)), "")
                ?.lineSequence()
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
                        .replace(EmojisUnicodeMapper.pattern) { matchResult ->
                            EmojisUnicodeMapper.map[matchResult.value.trim(':')] ?: matchResult.value
                        }
                        .trim()
                }
                ?.toList()
            return buildString {
                lines?.forEachIndexed { index, line ->
                    when {
                        line.startsWith('#') && (1..2).any { lines.getOrNull(index + it)?.startsWith("- ") == true } -> {
                            appendLine(line.trimStart('#').trim())
                        }
                        line.startsWith("- ") -> {
                            appendLine(
                                "- ${line.replace(Regex("([A-Z][a-z].*?[.!?]) ?(?=\$|[A-Z])"), "$1\n  ").drop(2).trim()}"
                            )
                        }
                    }
                }
            }.trim().takeUnless(String::isBlank)
        }

    fun String.cutToCharLimitWithLines(charLimit: Int): String {
        if (this.length <= charLimit) return this

        return buildString(charLimit) {
            var currentSize = 0

            for (line in this@cutToCharLimitWithLines.lineSequence()) {
                val prospectiveSize = currentSize + line.length
                if (prospectiveSize > charLimit) break
                appendLine(line)
                currentSize = prospectiveSize
            }
        }.trim()
    }
}
