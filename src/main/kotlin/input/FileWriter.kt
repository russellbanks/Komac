package input

import com.github.ajalt.mordant.terminal.Terminal
import utils.yesNoMenu
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
                createDirectoryIfNecessary(directory, files, terminal)
            }
        } while (directory?.isDirectory != true)
    }

    private fun createDirectoryIfNecessary(directory: File?, files: List<Pair<String, String?>>, terminal: Terminal) {
        with(terminal) {
            warning("The directory entered does not exist. Would you like to create it?")
            if (yesNoMenu(default = true)) {
                if (directory?.mkdirs() == true) {
                    success("Successfully created ${directory.path}")
                    writeFilesToDirectory(directory, files, terminal)
                } else {
                    danger("Failed to create ${directory?.path}")
                }
            }
        }
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
