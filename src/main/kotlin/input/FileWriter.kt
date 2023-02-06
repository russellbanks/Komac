package input

import com.github.ajalt.mordant.terminal.Terminal
import java.io.File

object FileWriter {
    fun writeFiles(files: List<Pair<String, String?>>, terminal: Terminal) = with(terminal) {
        do {
            println()
            println(colors.brightYellow("Enter a directory to write the files to:"))
            val directory = prompt("Directory")?.let { File(it) }
            if (directory?.isDirectory == true) {
                writeFilesToDirectory(directory, files, terminal)
            } else {
                warning("The directory entered is not a valid directory")
            }
        } while (directory?.isDirectory != true)
    }

    fun writeFilesToDirectory(directory: File, files: List<Pair<String, String?>>, terminal: Terminal) {
        files.forEach { file ->
            file.second?.let { manifestText ->
                writeFileToDirectory(directory, file.first, manifestText, terminal)
            }
        }
    }

    private fun writeFileToDirectory(directory: File, fileName: String, fileText: String, terminal: Terminal) {
        File(directory, fileName).apply {
            writeText(fileText.replace("\n", "\r\n"))
            if (exists()) {
                terminal.success("Successfully written $name to ${directory.path}")
            } else {
                terminal.danger("Failed to write $name")
            }
        }
    }
}
