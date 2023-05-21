package input

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import input.menu.yesNoMenu
import java.io.File

object FileWriter {
    fun writeFiles(files: Map<String, String?>, terminal: Terminal) = with(terminal) {
        do {
            println()
            println(colors.brightYellow("Enter a directory to write the files to:"))
            val directory = prompt("Directory")?.let(::File) ?: throw ProgramResult(ExitCode.CtrlC)
            if (directory.isDirectory) {
                writeFilesToDirectory(directory, files, terminal)
            } else {
                createDirectoryIfNecessary(directory, files, terminal)
            }
        } while (!directory.isDirectory)
    }

    private fun createDirectoryIfNecessary(directory: File, files: Map<String, String?>, terminal: Terminal) {
        with(terminal) {
            warning("The directory entered does not exist. Would you like to create it?")
            if (yesNoMenu(default = true).prompt()) {
                if (directory.mkdirs()) {
                    success("Successfully created ${directory.path}")
                    writeFilesToDirectory(directory, files, terminal)
                } else {
                    danger("Failed to create ${directory.path}")
                }
            }
        }
    }

    fun writeFilesToDirectory(directory: File, files: Map<String, String?>, terminal: Terminal) {
        for ((key, value) in files) {
            value?.let { manifestText ->
                writeFileToDirectory(directory, key, manifestText, terminal)
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
