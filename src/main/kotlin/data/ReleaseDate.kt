package data

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Pattern
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import java.util.Locale

object ReleaseDate : KoinComponent {
    fun Terminal.releaseDatePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightYellow(Prompts.releaseDateInfo))
            installerManifestData.releaseDate = prompt(brightWhite(PromptType.ReleaseDate.toString()))?.trim()
            val (releaseDateValid, error) = isReleaseDateValid(installerManifestData.releaseDate)
            error?.let { println(red(it)) }
            println()
        } while (releaseDateValid != Validation.Success)
    }

    fun isReleaseDateValid(releaseDate: String?): Pair<Validation, String?> {
        if (!releaseDate.isNullOrBlank()) {
            try {
                LocalDate.parse(releaseDate, DateTimeFormatter.ofPattern(Pattern.releaseDate, Locale.getDefault()))
            } catch (dateTimeParseException: DateTimeParseException) {
                return Validation.InvalidReleaseDate to Errors.invalidReleaseDate(dateTimeParseException)
            }
        }
        return Validation.Success to null
    }
}
