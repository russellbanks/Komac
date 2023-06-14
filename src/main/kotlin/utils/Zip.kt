package utils

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import io.ExitCode
import io.Prompts
import io.menu.checkMenu
import io.menu.radioMenu
import io.menu.yesNoMenu
import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import okio.FileSystem
import okio.Path
import okio.Path.Companion.toPath
import okio.buffer
import okio.openZip
import schemas.manifest.InstallerManifest
import utils.msix.MsixBundle

class Zip(zip: Path, fileSystem: FileSystem = FileSystem.SYSTEM) {
    var nestedInstallerType: InstallerManifest.InstallerType? = null
    var nestedInstallerFiles: List<InstallerManifest.NestedInstallerFiles>? = null

    private val validExtensionsList = listOf(
        InstallerManifest.InstallerType.MSIX.toString(),
        InstallerManifest.InstallerType.APPX.toString(),
        InstallerManifest.InstallerType.MSI.toString(),
        InstallerManifest.InstallerType.EXE.toString(),
        InstallerManifest.InstallerType.ZIP.toString(),
        MsixBundle.msixBundleConst,
        MsixBundle.appxBundleConst,
    )

    private val zipFileSystem = fileSystem.openZip(zip)

    private val identifiedFiles = zipFileSystem.listRecursively("/".toPath())
        .filter { zipEntry -> zipEntry.extension.lowercase() in validExtensionsList }
        .toList()

    private val installerTypeCounts = validExtensionsList.associateWith { validExtension ->
        identifiedFiles.count { zipEntry ->
            zipEntry.extension.lowercase() == validExtensionsList.find { it == validExtension }
        }
    }

    init {
        require(zip.extension.lowercase() == InstallerManifest.InstallerType.ZIP.toString()) {
            "File must be a ${InstallerManifest.InstallerType.ZIP}"
        }
    }

    fun prompt(terminal: Terminal): Unit = with(terminal) {
        if (installerTypeCounts.count { it.value == 1 } == 1) {
            nestedInstallerFiles = listOf(
                InstallerManifest.NestedInstallerFiles(relativeFilePath = identifiedFiles.first().toString())
            )
            nestedInstallerType = terminal.nestedInstallerTypePrompt(
                chosenZipEntries = listOf(identifiedFiles.first()),
                zipFileSystem = zipFileSystem
            )
            if (nestedInstallerType == InstallerManifest.InstallerType.PORTABLE) {
                nestedInstallerFiles = nestedInstallerFiles?.map {
                    it.copy(portableCommandAlias = terminal.portableCommandAliasPrompt(it.relativeFilePath))
                }
            }
        } else {
            if (installerTypeCounts.count { it.value != 0 && it.value <= 20 } == 1) {
                val chosenZipEntries = zipEntrySelectionPrompt(identifiedFiles)
                nestedInstallerFiles = chosenZipEntries.map {
                    InstallerManifest.NestedInstallerFiles(relativeFilePath = it.name)
                }
                nestedInstallerType = nestedInstallerTypePrompt(
                    chosenZipEntries = chosenZipEntries,
                    zipFileSystem = zipFileSystem
                )
                if (nestedInstallerType == InstallerManifest.InstallerType.PORTABLE) {
                    nestedInstallerFiles = nestedInstallerFiles?.map {
                        it.copy(portableCommandAlias = portableCommandAliasPrompt(it.relativeFilePath))
                    }
                }
            } else {
                nestedInstallersPrompt()
                nestedInstallerFiles?.let { nestedInstallerFiles ->
                    nestedInstallerType = nestedInstallerTypePrompt(
                        nestedInstallerFiles.map { it.relativeFilePath.toPath() },
                        zipFileSystem
                    )
                }
            }
        }
    }

    private fun Terminal.nestedInstallersPrompt() {
        do {
            do {
                println(colors.brightGreen("${Prompts.required} Enter the relative nested installer path"))
                info("Example: dart-sdk\\bin\\dart.exe")
                val input = prompt(
                    InstallerManifest.NestedInstallerFiles::relativeFilePath.name
                        .replaceFirstChar(Char::titlecase)
                        .replace("([A-Z])".toRegex(), " $1")
                        .trim()
                ) ?: throw ProgramResult(ExitCode.CtrlC)
                val error = isRelativeFilePathValid(input)?.also(::danger)
                var portableCommandAlias: String? = null
                if (nestedInstallerType == InstallerManifest.InstallerType.PORTABLE) {
                    println()
                    portableCommandAlias = portableCommandAliasPrompt()
                }
                if (error == null) {
                    nestedInstallerFiles = if (nestedInstallerFiles == null) {
                        listOf(
                            InstallerManifest.NestedInstallerFiles(
                                relativeFilePath = input,
                                portableCommandAlias = portableCommandAlias
                            )
                        )
                    } else {
                        nestedInstallerFiles?.plus(
                            InstallerManifest.NestedInstallerFiles(
                                relativeFilePath = input,
                                portableCommandAlias = portableCommandAlias
                            )
                        )
                    }
                }
            } while (error != null)
            info("Would you like to add another nested installer?")
            val shouldLoop = yesNoMenu(default = false).prompt()
        } while (shouldLoop)
    }

    private fun Terminal.portableCommandAliasPrompt(relativeFilePath: String? = null): String? {
        var portableCommandAlias: String?
        do {
            println(
                colors.brightYellow(
                    "${Prompts.optional} Enter the command line alias to be used for calling the package"
                )
            )
            info(if (relativeFilePath != null) "Installer: $relativeFilePath" else "Example: dart")
            portableCommandAlias = prompt(
                InstallerManifest.NestedInstallerFiles::portableCommandAlias.name
                    .replaceFirstChar(Char::titlecase)
                    .replace("([A-Z])".toRegex(), " $1")
                    .trim()
            )?.trim()
            val error = isPortableCommandAliasValid(portableCommandAlias)?.also(::danger)
            println()
        } while (error != null)
        return portableCommandAlias.takeIf { it?.isNotBlank() == true }
    }

    private fun isPortableCommandAliasValid(portableCommandAlias: String?): String? = when {
        portableCommandAlias.isNullOrBlank() -> null
        portableCommandAlias.length > portableCommandAliasMaxLength -> {
            Errors.invalidLength(min = portableCommandAliasMinLength, max = portableCommandAliasMaxLength)
        }
        else -> null
    }

    private fun isRelativeFilePathValid(relativeFilePath: String): String? = when {
        relativeFilePath.isBlank() -> Errors.blankInput(
            InstallerManifest.NestedInstallerFiles::relativeFilePath.name
                .replaceFirstChar(Char::titlecase)
                .replace("([A-Z])".toRegex(), " $1")
                .trim()
        )
        relativeFilePath.length > relativeFilePathMaxLength -> {
            Errors.invalidLength(min = relativeFilePathMinLength, max = relativeFilePathMaxLength)
        }
        else -> null
    }

    private fun Terminal.zipEntrySelectionPrompt(zipPaths: List<Path>): List<Path> = generateSequence {
        println(colors.brightGreen("${Prompts.required} Select files to use"))
        val chosenZipEntries = checkMenu<Path> {
            items = zipPaths
        }.prompt()

        chosenZipEntries.ifEmpty {
            println()
            danger("You have not chosen any nested files")
            danger("Please select at least one nested file")
            println()
            null
        }
    }.first()

    private fun Terminal.nestedInstallerTypePrompt(
        chosenZipEntries: List<Path>,
        zipFileSystem: FileSystem,
        fileSystem: FileSystem = FileSystem.SYSTEM,
        tempDirectory: Path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY
    ): InstallerManifest.InstallerType {
        val smallestEntry = chosenZipEntries.minBy { zipFileSystem.metadata(it).size ?: Long.MAX_VALUE }
        val formattedDate = Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).run {
            "$year.$monthNumber.$dayOfMonth-$hour.$minute.$second"
        }
        val tempFile = tempDirectory / "${smallestEntry.name.removeSuffix(".${smallestEntry.extension}")} - $formattedDate.${smallestEntry.extension}"
        zipFileSystem.source(smallestEntry).use { source ->
            fileSystem.sink(tempFile, mustCreate = true).buffer().use { bufferedSink ->
                bufferedSink.writeAll(source)
            }
        }
        val installerType = FileAnalyser(tempFile).installerType
        fileSystem.delete(tempFile)
        return if (installerType == null) {
            println(colors.brightGreen("${Prompts.required} Select the nested installer type"))
            radioMenu {
                items = listOf(
                    InstallerManifest.InstallerType.EXE,
                    InstallerManifest.InstallerType.PORTABLE
                )
            }.prompt() as InstallerManifest.InstallerType
        } else {
            installerType
        }
    }

    companion object {
        private const val relativeFilePathMinLength = 1
        private const val relativeFilePathMaxLength = 512
        private const val portableCommandAliasMinLength = 1
        private const val portableCommandAliasMaxLength = 40
    }
}
