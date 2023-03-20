package utils

import io.ktor.http.Url
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object FileUtils {
    suspend fun createTempFile(identifier: String, version: String, url: Url): File {
        return withContext(Dispatchers.IO) {
            val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
            File.createTempFile("$identifier v$version - $formattedDate", ".${url.getExtension()}}")
        }
    }
}
