package input

import com.github.ajalt.mordant.terminal.Terminal
import java.io.File

object FileWriter {
    fun Terminal.writeFiles(files: List<Pair<String, String?>>) {
        do {
            println()
            println(colors.brightYellow("Enter a directory to write the files to:"))
            val directory = prompt("Directory")?.let { File(it) }
            if (directory?.isDirectory == true) {
                writeFilesToDirectory(directory, files)
            } else {
                warning("The directory entered is not a valid directory")
            }
        } while (directory?.isDirectory != true)
    }

    private fun Terminal.writeFilesToDirectory(directory: File, files: List<Pair<String, String?>>) {
        files.forEach { file ->
            file.second?.let { manifestText ->
                writeFileToDirectory(directory, file.first, manifestText)
            }
        }
    }

    private fun Terminal.writeFileToDirectory(directory: File, fileName: String, fileText: String) {
        File(directory, fileName).apply {
            writeText(fileText.replace("\n", "\r\n"))
            if (exists()) {
                success("Successfully written $name to ${directory.path}")
            } else {
                danger("Failed to write $name")
            }
        }
    }
}
