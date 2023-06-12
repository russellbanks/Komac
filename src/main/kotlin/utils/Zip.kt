package utils

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import io.ExitCode
import io.Prompts
import io.menu.checkMenu
import io.menu.radioMenu
import okio.FileSystem
import okio.Path
import okio.Path.Companion.toPath
import okio.buffer
import okio.openZip
import schemas.manifest.InstallerManifest
import utils.msi.Msi
import utils.msix.MsixBundle

class Zip(zip: Path, fileSystem: FileSystem = FileSystem.SYSTEM, terminal: Terminal) {
    var nestedInstallerType: InstallerManifest.NestedInstallerType? = null
    var nestedInstallerFiles: List<InstallerManifest.NestedInstallerFiles>? = null
    private var installerTypeCounts: Map<String, Int>

    init {
        require(zip.extension == InstallerManifest.InstallerType.ZIP.toString()) {
            "File must be a ${InstallerManifest.InstallerType.ZIP}"
        }
        val validExtensionsList = listOf(
            InstallerManifest.NestedInstallerType.MSIX.toString(),
            InstallerManifest.NestedInstallerType.APPX.toString(),
            InstallerManifest.NestedInstallerType.MSI.toString(),
            InstallerManifest.NestedInstallerType.EXE.toString(),
            InstallerManifest.NestedInstallerType.ZIP.toString(),
            MsixBundle.msixBundleConst,
            MsixBundle.appxBundleConst,
        )
        val zipFileSystem = fileSystem.openZip(zip)
        val identifiedFiles = zipFileSystem.listRecursively("".toPath())
            .filter { zipEntry -> zipEntry.extension.lowercase() in validExtensionsList }
            .toList()
        installerTypeCounts = validExtensionsList.associateWith { validExtension ->
            identifiedFiles.count { zipEntry ->
                zipEntry.extension.lowercase() == validExtensionsList.find { it == validExtension }
            }
        }
        if (installerTypeCounts.count { it.value == 1 } == 1) {
            nestedInstallerFiles = listOf(
                InstallerManifest.NestedInstallerFiles(relativeFilePath = identifiedFiles.first().toString())
            )
            nestedInstallerType = terminal.nestedInstallerTypePrompt(
                chosenZipEntries = listOf(identifiedFiles.first()),
                zipFileSystem = zipFileSystem
            )
            if (nestedInstallerType == InstallerManifest.NestedInstallerType.PORTABLE) {
                nestedInstallerFiles = nestedInstallerFiles?.map {
                    it.copy(portableCommandAlias = terminal.portableCommandAliasPrompt(it.relativeFilePath))
                }
            }
        } else {
            if (installerTypeCounts.count { it.value != 0 && it.value <= 5 } == 1) {
                terminal.zipEntrySelectionPrompt(identifiedFiles).let { chosenZipEntries ->
                    nestedInstallerFiles = chosenZipEntries.map {
                        InstallerManifest.NestedInstallerFiles(relativeFilePath = it.name)
                    }
                    nestedInstallerType = terminal.nestedInstallerTypePrompt(
                        chosenZipEntries = chosenZipEntries,
                        zipFileSystem = zipFileSystem
                    )
                    if (nestedInstallerType == InstallerManifest.NestedInstallerType.PORTABLE) {
                        nestedInstallerFiles = nestedInstallerFiles?.map {
                            it.copy(portableCommandAlias = terminal.portableCommandAliasPrompt(it.relativeFilePath))
                        }
                    }
                }
            } else {
                terminal.nestedInstallersPrompt()
                nestedInstallerFiles?.let { nestedInstallerFiles ->
                    nestedInstallerType = terminal.nestedInstallerTypePrompt(
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
                if (nestedInstallerType == InstallerManifest.NestedInstallerType.PORTABLE) {
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
            val shouldLoop = YesNoPrompt(
                prompt = "${Prompts.optional} Would you like to add another nested installer?",
                terminal = this
            ).ask()
        } while (shouldLoop == true)
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

    private fun Terminal.zipEntrySelectionPrompt(zipPaths: List<Path>): List<Path> {
        var chosenZipEntries: List<Path>
        do {
            println(colors.brightGreen("${Prompts.required} Select files to use"))
            chosenZipEntries = checkMenu<Path> {
                items = zipPaths
            }.prompt()
            if (chosenZipEntries.isEmpty()) {
                println()
                danger("You have not chosen any nested files")
                danger("Please select at least one nested file")
            }
            println()
        } while (chosenZipEntries.isEmpty())
        return chosenZipEntries
    }


    @OptIn(ExperimentalStdlibApi::class)
    private fun Terminal.nestedInstallerTypePrompt(
        chosenZipEntries: List<Path>,
        zipFileSystem: FileSystem,
        fileSystem: FileSystem = FileSystem.SYSTEM,
        tempDirectory: Path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY
    ): InstallerManifest.NestedInstallerType {
        val smallestEntry = chosenZipEntries.minBy { zipFileSystem.metadata(it).size ?: Long.MAX_VALUE }
        return when (smallestEntry.extension.lowercase()) {
            InstallerManifest.NestedInstallerType.MSIX.toString(), MsixBundle.msixBundleConst -> {
                InstallerManifest.NestedInstallerType.MSIX
            }
            InstallerManifest.NestedInstallerType.APPX.toString(), MsixBundle.appxBundleConst -> {
                InstallerManifest.NestedInstallerType.APPX
            }
            InstallerManifest.NestedInstallerType.ZIP.toString() -> {
                InstallerManifest.NestedInstallerType.ZIP
            }
            InstallerManifest.NestedInstallerType.MSI.toString() -> {
                val tempFile = tempDirectory / "${smallestEntry.name}.${InstallerManifest.NestedInstallerType.MSI}"
                zipFileSystem.source(smallestEntry).use { source ->
                    fileSystem.sink(tempFile, mustCreate = true).buffer().use { it.writeAll(source) }
                }
                if (Msi(tempFile).isWix.also { fileSystem.delete(tempFile) }) {
                    InstallerManifest.NestedInstallerType.WIX
                } else {
                    InstallerManifest.NestedInstallerType.MSI
                }
            }
            InstallerManifest.NestedInstallerType.EXE.toString() -> {
                val exeNestedTypes = listOf(
                    InstallerManifest.NestedInstallerType.EXE,
                    InstallerManifest.NestedInstallerType.BURN,
                    InstallerManifest.NestedInstallerType.INNO,
                    InstallerManifest.NestedInstallerType.NULLSOFT,
                    InstallerManifest.NestedInstallerType.PORTABLE
                )
                println(colors.brightGreen("${Prompts.required} Enter the nested installer type"))
                info("Options: ${exeNestedTypes.joinToString()}")
                radioMenu<InstallerManifest.NestedInstallerType> {
                    items = exeNestedTypes
                }.prompt() as InstallerManifest.NestedInstallerType
            }
            else -> radioMenu<InstallerManifest.NestedInstallerType> {
                items = InstallerManifest.NestedInstallerType.entries
            }.prompt() as InstallerManifest.NestedInstallerType
        }.also { println() }
    }

    companion object {
        private const val relativeFilePathMinLength = 1
        private const val relativeFilePathMaxLength = 512
        private const val portableCommandAliasMinLength = 1
        private const val portableCommandAliasMaxLength = 40
    }
}
