package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import kotlin.random.Random

object ReleaseDate : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.releaseDatePrompt() {
        do {
            println(brightYellow(releaseDateInfo))
            println(cyan(releaseDateExample))
            getPreviousValue()?.let { println(gray("Previous release date: $it")) }
            val input = prompt(brightWhite(PromptType.ReleaseDate.toString()))?.trim()
            val (releaseDateValid, error) = isReleaseDateValid(input)
            error?.let { println(brightRed(it)) }
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

    private fun getPreviousValue(): LocalDate? {
        return previousManifestData.remoteInstallerData?.let {
            it.releaseDate ?: it.installers[installerManifestData.installers.size].releaseDate
        }
    }

    private fun generateRandomLocalDate(startInclusive: LocalDate, endExclusive: LocalDate): LocalDate {
        val startEpochDay = startInclusive.toEpochDay()
        val endEpochDay = endExclusive.toEpochDay()
        val randomDay = Random.nextLong(startEpochDay, endEpochDay)
        return LocalDate.ofEpochDay(randomDay)
    }

    private fun invalidReleaseDate(dateTimeParseException: DateTimeParseException): String {
        return "${Errors.error} Invalid Date - ${
        dateTimeParseException.cause?.message
            ?: dateTimeParseException.message
            ?: "Input could not be resolved to a date"
        }"
    }

    private val releaseDateExample = "Example: ${generateRandomLocalDate(
        startInclusive = LocalDate.now().minusYears(/* yearsToSubtract = */ 30),
        endExclusive = LocalDate.now()
    )}"

    const val releaseDatePattern = "yyyy-MM-dd"
    private const val releaseDateInfo = "${Prompts.optional} Enter the application release date in the format " +
        releaseDatePattern
}
