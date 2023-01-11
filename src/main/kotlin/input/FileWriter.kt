package input

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import java.io.File

object FileWriter {
    fun Terminal.writeFiles(files: List<Pair<String, String?>>) {
        do {
            println()
            println(brightYellow("Enter a directory to write the files to:"))
            val directory = prompt(brightWhite("Directory"))?.let { File(it) }
            if (directory?.isDirectory == true) {
                writeFilesToDirectory(directory, files)
            } else {
                println("The directory entered is not a valid directory")
            }
        } while (directory?.isDirectory != true)
    }

    private fun writeFilesToDirectory(directory: File, files: List<Pair<String, String?>>) {
        files.forEach { file ->
            file.second?.let { manifestText ->
                writeFileToDirectory(directory, file.first, manifestText)
            }
        }
    }

    private fun writeFileToDirectory(directory: File, fileName: String, fileText: String) {
        File(directory, fileName).apply {
            writeText(fileText.replace("\n", "\r\n"))
            if (exists()) {
                println(brightGreen("Successfully written $name to ${directory.path}"))
            } else {
                println(red("Failed to write $name"))
            }
        }
    }
}
