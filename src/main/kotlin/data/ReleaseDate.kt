package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException

object ReleaseDate : KoinComponent {
    fun Terminal.releaseDatePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightYellow(releaseDateInfo))
            val input = prompt(brightWhite(PromptType.ReleaseDate.toString()))?.trim()
            val (releaseDateValid, error) = isReleaseDateValid(input)
            error?.let { println(red(it)) }
            if (releaseDateValid == Validation.Success && !input.isNullOrBlank()) {
                installerManifestData.releaseDate = LocalDate.parse(
                    input,
                    DateTimeFormatter.ofPattern(releaseDatePattern)
                )
            }
            println()
        } while (releaseDateValid != Validation.Success)
    }

    fun isReleaseDateValid(releaseDate: String?): Pair<Validation, String?> {
        if (!releaseDate.isNullOrBlank()) {
            try {
                LocalDate.parse(releaseDate, DateTimeFormatter.ofPattern(releaseDatePattern))
            } catch (dateTimeParseException: DateTimeParseException) {
                return Validation.InvalidReleaseDate to invalidReleaseDate(dateTimeParseException)
            }
        }
        return Validation.Success to null
    }

    private fun invalidReleaseDate(dateTimeParseException: DateTimeParseException): String {
        return "${Errors.error} Invalid Date - ${
        dateTimeParseException.cause?.message
            ?: dateTimeParseException.message
            ?: "Input could not be resolved to a date"
        }"
    }

    const val releaseDatePattern = "yyyy-MM-dd"
    private const val releaseDateInfo = "${Prompts.optional} Enter the application release date in the format " +
        "$releaseDatePattern. Example: 2022-11-17"
}
