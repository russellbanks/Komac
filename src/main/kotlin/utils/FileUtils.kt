package utils

import io.ktor.http.Url
import okio.Path
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object FileUtils {
    fun createTempFile(
        identifier: String,
        version: String,
        url: Url,
        tempDirectory: Path
    ): Path {
        val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
        return tempDirectory / "$identifier v$version - $formattedDate.${url.extension}"
    }
}
