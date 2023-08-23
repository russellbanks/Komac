package utils

import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.Theme

object ManifestUtils {
    /**
     * Takes a raw manifest string and returns a sequence of formatted lines for the manifest.
     * The function processes each line of the raw manifest string and formats it based on the first character of the
     * line:
     * - If the line starts with "#" character, the line is formatted in green.
     * - If the first character of the line is a letter, the line is split into two parts based on the first colon
     * character (":"),
     *   and the first part is formatted using the info colour and the second part (if any) is left
     *   unformatted.
     * - Otherwise, the line is left unchanged.
     *
     * @param rawString the raw manifest string to be formatted.
     * @param theme the [Theme] instance to use for formatting the manifest.
     * @return a sequence of formatted lines for the manifest.
     */
    fun formattedManifestLinesSequence(rawString: String, theme: Theme): Sequence<String> {
        return rawString.lineSequence().map { line ->
            when {
                line.startsWith('#') -> TextColors.green(line)
                line.firstOrNull()?.isLetter() == true -> {
                    val (key, value) = line.split(':', limit = 2)
                    "${theme.info(key)}:$value"
                }
                else -> line
            }
        }
    }

    /**
     * Returns a new [String] in which all occurrences of any version in [previousVersions] are replaced with the
     * [newVersion]. If [previousVersions] is null or empty, this function returns the original [String] itself.
     *
     * @param previousVersions A list of [String] versions to replace in the [String].
     * @param newVersion The new [String] version to replace the old versions with.
     * @return The updated [String] with the [newVersion] in place of the old versions.
     */
    fun String.updateVersionInString(previousVersions: List<String>?, newVersion: String): String {
        // If previousVersions is null or empty, return the original string
        if (previousVersions.isNullOrEmpty()) return this

        // Create a regex pattern from the previous versions
        val pattern = previousVersions.joinToString("|") { Regex.escape(it) }.toRegex()

        // Replace all occurrences of the pattern with the new version
        return replace(pattern, newVersion)
    }
}
