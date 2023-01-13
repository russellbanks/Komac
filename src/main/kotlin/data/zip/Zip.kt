package data.zip

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextStyles.bold
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.msi.Msi
import data.msix.MsixBundle
import input.Polar
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.InstallerManifest
import schemas.TerminalInstance
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipFile
import java.util.zip.ZipOutputStream


class Zip(zip: File) : KoinComponent {
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
            InstallerManifest.Installer.NestedInstallerType.EXE.toString(),
            InstallerManifest.Installer.NestedInstallerType.ZIP.toString(),
            MsixBundle.msixBundleConst,
            MsixBundle.appxBundleConst,
        )
        ZipFile(zip).use { zipFile ->
            zipFile.entries()
                .asSequence()
                .toList()
                .filter { zipEntry -> zipEntry.name.substringAfterLast(".").lowercase() in validExtensionsList }
                .also { zipEntries ->
                    installerTypeCounts = validExtensionsList.associateWith { validExtension ->
                        zipEntries.count { zipEntry ->
                            val extension = zipEntry.name.substringAfterLast(".").lowercase()
                            extension == validExtensionsList.find { it == validExtension }
                        }
                    }
                }
                .let { zipEntries ->
                    if (installerTypeCounts.count { it.value == 1 } == 1) {
                        nestedInstallerFiles = listOf(
                            InstallerManifest.Installer.NestedInstallerFiles(relativeFilePath = zipEntries.first().name)
                        )
                    } else {
                        if (installerTypeCounts.count { it.value != 0 } == 1) {
                            with(get<TerminalInstance>().terminal) {
                                zipEntrySelectionPrompt(zipEntries).let { chosenZipEntries ->
                                    nestedInstallerFiles = chosenZipEntries.map {
                                        InstallerManifest.Installer.NestedInstallerFiles(relativeFilePath = it.name)
                                    }
                                    nestedInstallerTypePrompt(chosenZipEntries)
                                }
                            }
                        } else {
                            // Prompt for nested installer path
                        }
                    }
                }
        }
    }

    private fun Terminal.zipEntrySelectionPrompt(zipEntries: List<ZipEntry>): MutableList<ZipEntry> {
        val chosenZipEntries: MutableList<ZipEntry> = mutableListOf()
        do {
            zipEntries.forEachIndexed { index, zipEntry ->
                println(
                    verticalLayout {
                        cell(brightGreen("${Prompts.required} Would you like to use ${zipEntry.name}?"))
                        cell(
                            cyan(
                                buildString {
                                    append("Detected ")
                                    append(zipEntry.name.substringAfterLast(".").lowercase())
                                    append(" ")
                                    append(index.inc())
                                    append("/")
                                    append(zipEntries.size)
                                }
                            )
                        )
                        Polar.values().forEach {
                            cell(brightWhite("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                        }
                    }
                )
                prompt(
                    prompt = brightWhite(Prompts.enterChoice),
                    convert = {
                        when (it.firstOrNull()?.lowercase()) {
                            Polar.Yes.name.first().lowercase() -> ConversionResult.Valid(Polar.Yes)
                            Polar.No.name.first().lowercase() -> ConversionResult.Valid(Polar.No)
                            else -> ConversionResult.Invalid("Invalid choice")
                        }
                    }
                ).let {
                    if (it == Polar.Yes) {
                        chosenZipEntries.add(zipEntry)
                    }
                }
            }
            if (chosenZipEntries.isEmpty()) {
                val redAndBold = brightRed + bold
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
        chosenZipEntries: List<ZipEntry>
    ): InstallerManifest.Installer.NestedInstallerType {
        return when (chosenZipEntries.first().name) {
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
                // Check if wix
                InstallerManifest.Installer.NestedInstallerType.MSI
            }
            InstallerManifest.Installer.NestedInstallerType.EXE.toString() -> {
                val exeNestedTypes = listOf(
                    InstallerManifest.Installer.NestedInstallerType.EXE,
                    InstallerManifest.Installer.NestedInstallerType.BURN,
                    InstallerManifest.Installer.NestedInstallerType.INNO,
                    InstallerManifest.Installer.NestedInstallerType.NULLSOFT,
                    InstallerManifest.Installer.NestedInstallerType.PORTABLE
                )
                println(brightGreen("${Prompts.required} Enter the nested installer type"))
                println(cyan("Options: ${exeNestedTypes.joinToString(", ")}"))
                val input = prompt(
                    prompt = Prompts.enterChoice,
                    convert = { string ->
                        if (string.lowercase() in exeNestedTypes.map { it.toString() }) {
                            ConversionResult.Valid(
                                InstallerManifest.Installer.NestedInstallerType.valueOf(string.uppercase())
                            )
                        } else {
                            ConversionResult.Invalid(
                                "${Errors.error} - Value must exist in the enum - ${exeNestedTypes.joinToString(", ")}"
                            )
                        }
                    }
                )
                InstallerManifest.Installer.NestedInstallerType.EXE
            }
            else -> {
                TODO()
            }
        }
    }
}
