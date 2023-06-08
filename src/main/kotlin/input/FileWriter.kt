package input

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import input.menu.yesNoMenu
import okio.FileSystem
import okio.Path
import okio.Path.Companion.toPath

object FileWriter {
    fun writeFiles(files: Map<String, String?>, terminal: Terminal) = with(terminal) {
        do {
            println()
            println(colors.brightYellow("Enter a directory to write the files to:"))
            val directory = prompt("Directory")?.toPath() ?: throw ProgramResult(ExitCode.CtrlC)
            if (FileSystem.SYSTEM.metadata(directory).isDirectory) {
                writeFilesToDirectory(directory, files, terminal)
            } else {
                createDirectoryIfNecessary(directory, files, terminal)
            }
        } while (!FileSystem.SYSTEM.metadata(directory).isDirectory)
    }

    private fun createDirectoryIfNecessary(directory: Path, files: Map<String, String?>, terminal: Terminal) {
        with(terminal) {
            warning("The directory entered does not exist. Would you like to create it?")
            if (yesNoMenu(default = true).prompt()) {
                FileSystem.SYSTEM.createDirectory(directory)
                success("Successfully created $directory")
                writeFilesToDirectory(directory, files, terminal)
            }
        }
    }

    fun writeFilesToDirectory(directory: Path, files: Map<String, String?>, terminal: Terminal) {
        for ((fileName, fileText) in files) {
            if (fileText != null) {
                val file = (directory / fileName).apply {
                    FileSystem.SYSTEM.write(this) {
                        use { writeUtf8(fileText.replace("\n", "\r\n")) }
                    }
                }
                if (FileSystem.SYSTEM.exists(file)) {
                    terminal.success("Successfully written ${file.name} to $directory")
                } else {
                    terminal.danger("Failed to write ${file.name}")
                }
            }
        }
    }
}
