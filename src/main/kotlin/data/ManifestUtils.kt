package data

import com.github.ajalt.mordant.terminal.TerminalColors
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

object ManifestUtils {
    fun formattedManifestLinesFlow(rawString: String, colors: TerminalColors): Flow<String> = flow {
        rawString.lines().forEach { line ->
            when {
                line.startsWith("#") -> emit(colors.green(line))
                line.firstOrNull()?.isLetter() == true -> {
                    val part = line.split(":", limit = 2)
                    emit("${colors.info(part.first())}${part.getOrNull(1)?.let { ":$it" } ?: ""}")
                }
                line.startsWith("-") -> emit(line)
                else -> emit(line)
            }
        }
    }
}
