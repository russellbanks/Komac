package detection.files

import Errors
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import com.github.ajalt.mordant.terminal.YesNoPrompt
import detection.files.msi.Msi
import detection.files.msix.MsixBundle
import input.Prompts
import okio.Path.Companion.toOkioPath
import schemas.manifest.InstallerManifest
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipFile

class Zip(zip: File, terminal: Terminal) {
    var nestedInstallerType: InstallerManifest.Installer.NestedInstallerType? = null
    var nestedInstallerFiles: List<InstallerManifest.Installer.NestedInstallerFiles>? = null
    private var installerTypeCounts: Map<String, Int>

    init {
        require(zip.extension.lowercase() == InstallerManifest.InstallerType.ZIP.toString()) {
            "File must be a ${InstallerManifest.InstallerType.ZIP}"
        }
        val validExtensionsList = listOf(
            InstallerManifest.Installer.NestedInstallerType.MSIX.toString(),
            InstallerManifest.Installer.NestedInstallerType.APPX.toString(),
            InstallerManifest.Installer.NestedInstallerType.MSI.toString(),
            InstallerManifest.Installer.NestedInstallerType.EXE.toString(),
            InstallerManifest.Installer.NestedInstallerType.ZIP.toString(),
            MsixBundle.msixBundleConst,
            MsixBundle.appxBundleConst,
        )
        ZipFile(zip).use { zipFile ->
            val zipEntries = zipFile.entries()
                .asSequence()
                .toList()
                .filter { zipEntry -> zipEntry.name.substringAfterLast('.').lowercase() in validExtensionsList }
            installerTypeCounts = validExtensionsList.associateWith { validExtension ->
                zipEntries.count { zipEntry ->
                    val extension = zipEntry.name.substringAfterLast('.').lowercase()
                    extension == validExtensionsList.find { it == validExtension }
                }
            }
            with(terminal) {
                if (installerTypeCounts.count { it.value == 1 } == 1) {
                    let {
                        nestedInstallerFiles = listOf(
                            InstallerManifest.Installer.NestedInstallerFiles(relativeFilePath = zipEntries.first().name)
                        )
                        nestedInstallerType = nestedInstallerTypePrompt(
                            chosenZipEntries = listOf(zipEntries.first()),
                            zipFile = zipFile
                        )
                        if (nestedInstallerType == InstallerManifest.Installer.NestedInstallerType.PORTABLE) {
                            nestedInstallerFiles = nestedInstallerFiles?.map { nestedInstallerFIle ->
                                nestedInstallerFIle.copy(
                                    portableCommandAlias = portableCommandAliasPrompt(
                                        relativeFilePath = nestedInstallerFIle.relativeFilePath
                                    )
                                )
                            }
                        }
                    }
                } else {
                    if (installerTypeCounts.count { it.value != 0 && it.value <= 5 } == 1) {
                        zipEntrySelectionPrompt(zipEntries).let { chosenZipEntries ->
                            nestedInstallerFiles = chosenZipEntries.map {
                                InstallerManifest.Installer.NestedInstallerFiles(relativeFilePath = it.name)
                            }
                            nestedInstallerType = nestedInstallerTypePrompt(
                                chosenZipEntries = chosenZipEntries,
                                zipFile = zipFile
                            )
                            if (nestedInstallerType == InstallerManifest.Installer.NestedInstallerType.PORTABLE) {
                                nestedInstallerFiles = nestedInstallerFiles?.map {
                                    it.copy(portableCommandAlias = portableCommandAliasPrompt(it.relativeFilePath))
                                }
                            }
                        }
                    } else {
                        nestedInstallersPrompt()
                        nestedInstallerFiles?.let { nestedInstallerFiles ->
                            nestedInstallerType = nestedInstallerTypePrompt(
                                nestedInstallerFiles.map { zipFile.getEntry(it.relativeFilePath) },
                                zipFile
                            )
                        }
                    }
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
                    InstallerManifest.Installer.NestedInstallerFiles::relativeFilePath.name
                        .replaceFirstChar(Char::titlecase)
                        .replace("([A-Z])".toRegex(), " $1")
                        .trim()
                )
                val error = isRelativeFilePathValid(input)?.also { danger(it) }
                var portableCommandAlias: String? = null
                if (nestedInstallerType == InstallerManifest.Installer.NestedInstallerType.PORTABLE) {
                    println()
                    portableCommandAlias = portableCommandAliasPrompt()
                }
                if (error == null && input != null) {
                    nestedInstallerFiles = if (nestedInstallerFiles == null) {
                        listOf(
                            InstallerManifest.Installer.NestedInstallerFiles(
                                relativeFilePath = input,
                                portableCommandAlias = portableCommandAlias
                            )
                        )
                    } else {
                        nestedInstallerFiles?.plus(
                            InstallerManifest.Installer.NestedInstallerFiles(
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
                InstallerManifest.Installer.NestedInstallerFiles::portableCommandAlias.name
                    .replaceFirstChar(Char::titlecase)
                    .replace("([A-Z])".toRegex(), " $1")
                    .trim()
            )?.trim()
            val error = isPortableCommandAliasValid(portableCommandAlias)?.also { danger(it) }
            println()
        } while (error != null)
        return portableCommandAlias.takeIf { it?.isNotBlank() == true }
    }

    private fun isPortableCommandAliasValid(portableCommandAlias: String?): String? {
        return when {
            portableCommandAlias.isNullOrBlank() -> null
            portableCommandAlias.length > portableCommandAliasMaxLength -> {
                Errors.invalidLength(min = portableCommandAliasMinLength, max = portableCommandAliasMaxLength)
            }
            else -> null
        }
    }

    private fun isRelativeFilePathValid(relativeFilePath: String?): String? {
        return when {
            relativeFilePath.isNullOrBlank() -> Errors.blankInput(
                InstallerManifest.Installer.NestedInstallerFiles::relativeFilePath.name
                    .replaceFirstChar(Char::titlecase)
                    .replace("([A-Z])".toRegex(), " $1")
                    .trim()
            )
            relativeFilePath.length > relativeFilePathMaxLength -> {
                Errors.invalidLength(min = relativeFilePathMinLength, max = relativeFilePathMaxLength)
            }
            else -> null
        }
    }

    private fun Terminal.zipEntrySelectionPrompt(zipEntries: List<ZipEntry>): MutableList<ZipEntry> {
        val chosenZipEntries: MutableList<ZipEntry> = mutableListOf()
        do {
            zipEntries.forEachIndexed { index, zipEntry ->
                println(
                    verticalLayout {
                        cell(colors.brightGreen("${Prompts.required} Would you like to use ${zipEntry.name}?"))
                        cell(
                            colors.cyan(
                                buildString {
                                    append("Detected ")
                                    append(zipEntry.name.substringAfterLast('.').lowercase())
                                    append(" ")
                                    append(index.inc())
                                    append("/")
                                    append(zipEntries.size)
                                }
                            )
                        )
                    }
                )
                YesNoPrompt(prompt = Prompts.enterChoice, terminal = this).ask()?.let {
                    if (it) chosenZipEntries.add(zipEntry)
                }
            }
            if (chosenZipEntries.isEmpty()) {
                val redAndBold = colors.brightRed + colors.bold
                println(
                    verticalLayout {
                        cell("")
                        cell(redAndBold("You have not chosen any nested files"))
                        cell(redAndBold("Please select at least one nested file"))
                    }
                )
            }
            println()
        } while (chosenZipEntries.isEmpty())
        return chosenZipEntries
    }

    private fun Terminal.nestedInstallerTypePrompt(
        chosenZipEntries: List<ZipEntry>,
        zipFile: ZipFile
    ): InstallerManifest.Installer.NestedInstallerType {
        val smallestEntry = chosenZipEntries.minBy { it.size }
        return when (smallestEntry.name.substringAfterLast('.').lowercase()) {
            InstallerManifest.Installer.NestedInstallerType.MSIX.toString(), MsixBundle.msixBundleConst -> {
                InstallerManifest.Installer.NestedInstallerType.MSIX
            }
            InstallerManifest.Installer.NestedInstallerType.APPX.toString(), MsixBundle.appxBundleConst -> {
                InstallerManifest.Installer.NestedInstallerType.APPX
            }
            InstallerManifest.Installer.NestedInstallerType.ZIP.toString() -> {
                InstallerManifest.Installer.NestedInstallerType.ZIP
            }
            InstallerManifest.Installer.NestedInstallerType.MSI.toString() -> {
                val tempFile = File.createTempFile(
                    smallestEntry.name,
                    InstallerManifest.Installer.NestedInstallerType.MSI.toString()
                )
                zipFile.getInputStream(smallestEntry).use { input ->
                    tempFile.outputStream().use(input::copyTo)
                }
                if (Msi(tempFile.toOkioPath()).isWix.also { tempFile.delete() }) {
                    InstallerManifest.Installer.NestedInstallerType.WIX
                } else {
                    InstallerManifest.Installer.NestedInstallerType.MSI
                }
            }
            InstallerManifest.Installer.NestedInstallerType.EXE.toString() -> {
                val exeNestedTypes = listOf(
                    InstallerManifest.Installer.NestedInstallerType.EXE,
                    InstallerManifest.Installer.NestedInstallerType.BURN,
                    InstallerManifest.Installer.NestedInstallerType.INNO,
                    InstallerManifest.Installer.NestedInstallerType.NULLSOFT,
                    InstallerManifest.Installer.NestedInstallerType.PORTABLE
                ).map(InstallerManifest.Installer.NestedInstallerType::toString)
                println(colors.brightGreen("${Prompts.required} Enter the nested installer type"))
                info("Options: ${exeNestedTypes.joinToString()}")
                prompt(
                    prompt = Prompts.enterChoice,
                    convert = { string ->
                        if (string.lowercase() in exeNestedTypes) {
                            ConversionResult.Valid(
                                InstallerManifest.Installer.NestedInstallerType.valueOf(string.uppercase())
                            )
                        } else {
                            ConversionResult.Invalid(Errors.invalidEnum(enum = exeNestedTypes))
                        }
                    }
                ) ?: InstallerManifest.Installer.NestedInstallerType.EXE
            }
            else -> {
                val nestedInstallerTypes = InstallerManifest.Installer.NestedInstallerType
                    .values()
                    .map { it.toString() }
                prompt(
                    prompt = Prompts.enterChoice,
                    convert = { string ->
                        if (string.lowercase() in nestedInstallerTypes) {
                            ConversionResult.Valid(
                                InstallerManifest.Installer.NestedInstallerType.valueOf(string.uppercase())
                            )
                        } else {
                            ConversionResult.Invalid(Errors.invalidEnum(enum = nestedInstallerTypes))
                        }
                    }
                ) ?: InstallerManifest.Installer.NestedInstallerType.EXE
            }
        }.also { println() }
    }

    companion object {
        private const val relativeFilePathMinLength = 1
        private const val relativeFilePathMaxLength = 512
        private const val portableCommandAliasMinLength = 1
        private const val portableCommandAliasMaxLength = 40
    }
}
