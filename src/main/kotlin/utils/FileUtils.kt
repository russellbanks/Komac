package utils

import io.ktor.http.Url
import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import okio.Path

object FileUtils {
    fun createTempFile(
        identifier: String,
        version: String,
        url: Url,
        tempDirectory: Path
    ): Path {
        val formattedDate = Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).run {
            "$year.$monthNumber.$dayOfMonth-$hour.$minute.$second"
        }
        return tempDirectory / "$identifier v$version - $formattedDate.${url.extension}"
    }
}
