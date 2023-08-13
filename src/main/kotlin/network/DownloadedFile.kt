package network

import com.github.ajalt.mordant.animation.ProgressAnimation
import kotlinx.datetime.LocalDate
import okio.Path

data class DownloadedFile(
    val path: Path,
    val lastModified: LocalDate?,
    val fileDeletionHook: Thread,
    val progress: ProgressAnimation
) {
    fun removeFileDeletionHook() = Runtime.getRuntime().removeShutdownHook(fileDeletionHook)
}
