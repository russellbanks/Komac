package utils

import com.github.ajalt.mordant.terminal.TerminalColors

object ManifestUtils {
    /**
     * Takes a raw manifest string and a [TerminalColors] instance, and returns a sequence of formatted lines for the
     * manifest.
     * The function processes each line of the raw manifest string and formats it based on the first character of the
     * line:
     * - If the line starts with "#" character, the line is formatted in green color using the [TerminalColors.green]
     * function.
     * - If the first character of the line is a letter, the line is split into two parts based on the first colon
     * character (":"),
     *   and the first part is formatted using the [TerminalColors.info] function and the second part (if any) is left
     *   unformatted.
     * - Otherwise, the line is left unchanged.
     * Each formatted line is returned as a string in a sequence using the [yield] function.
     *
     * @param rawString the raw manifest string to be formatted.
     * @param colors the [TerminalColors] instance to use for formatting the manifest.
     * @return a sequence of formatted lines for the manifest.
     */
    fun formattedManifestLinesSequence(rawString: String, colors: TerminalColors): Sequence<String> = sequence {
        rawString.lines().forEach { line ->
            yield(
                when {
                    line.startsWith("#") -> colors.green(line)
                    line.firstOrNull()?.isLetter() == true -> {
                        val part = line.split(":", limit = 2)
                        "${colors.info(part.first())}${part.getOrNull(1)?.let { ":$it" }.orEmpty()}"
                    }
                    else -> line
                }
            )
        }
    }

    /**
     * Returns a new [String] in which the first occurrence of any version in [previousVersions] is replaced with the
     * [newVersion]. If [previousVersions] is null or empty, this function returns the original [String] itself.
     *
     * @param previousVersions A list of [String] versions to replace in the [String].
     * @param newVersion The new [String] version to replace the old version with.
     * @return The updated [String] with the [newVersion] in place of the old version.
     */
    fun String.updateVersionInString(previousVersions: List<String>?, newVersion: String) = previousVersions
        ?.joinToString("|") { it }
        ?.let { replaceFirst(it.toRegex(), newVersion) }
        ?: this
}
