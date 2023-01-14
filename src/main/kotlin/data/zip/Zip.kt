package data.zip

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
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
import schemas.InstallerSchema
import schemas.SchemasImpl
import schemas.TerminalInstance
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipFile

class Zip(zip: File) : KoinComponent {
    var nestedInstallerType: InstallerManifest.Installer.NestedInstallerType? = null
    var nestedInstallerFiles: List<InstallerManifest.Installer.NestedInstallerFiles>? = null
    private var installerTypeCounts: Map<String, Int>
    private val installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    private val nestedInstallerPropertiesSchema = installerSchema.definitions.nestedInstallerFiles.items.properties

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
        val terminal = get<TerminalInstance>().terminal
        ZipFile(zip).use { zipFile ->
            val zipEntries = zipFile.entries()
                .asSequence()
                .toList()
                .filter { zipEntry -> zipEntry.name.substringAfterLast(".").lowercase() in validExtensionsList }
            installerTypeCounts = validExtensionsList.associateWith { validExtension ->
                zipEntries.count { zipEntry ->
                    val extension = zipEntry.name.substringAfterLast(".").lowercase()
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
                            nestedInstallerFiles = nestedInstallerFiles?.map {
                                it.copy(portableCommandAlias = portableCommandAliasPrompt(it.relativeFilePath))
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
                println(brightGreen("${Prompts.required} Enter the relative nested installer path"))
                println(cyan("Example: dart-sdk\\bin\\dart.exe"))
                val input = prompt(
                    InstallerManifest.Installer.NestedInstallerFiles::relativeFilePath.name
                        .replaceFirstChar { it.titlecase() }
                        .replace(Regex("([A-Z])"), " $1").trim()
                )
                val error = isRelativeFilePathValid(input)?.also { println(brightRed(it)) }
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
            println(
                verticalLayout {
                    cell(brightYellow("${Prompts.optional} Would you like to add another nested installer?"))
                    cell(
                        cyan(
                            buildString {
                                append("You have added ")
                                append(nestedInstallerFiles?.size)
                                append(" nested installer")
                                if ((nestedInstallerFiles?.size ?: 0) > 1) append("s")
                            }
                        )
                    )
                    Polar.values().forEach {
                        cell(brightWhite("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                    }
                }
            )
            val shouldLoop = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                convert = {
                    when (it.firstOrNull()?.lowercase()) {
                        Polar.Yes.name.first().lowercase() -> ConversionResult.Valid(true)
                        Polar.No.name.first().lowercase() -> ConversionResult.Valid(false)
                        else -> ConversionResult.Invalid("Invalid choice")
                    }
                }
            )
        } while (shouldLoop == true)
    }

    private fun Terminal.portableCommandAliasPrompt(relativeFilePath: String? = null): String? {
        val portableCommandAliasSchema = nestedInstallerPropertiesSchema.portableCommandAlias
        var portableCommandAlias: String?
        do {
            println(
                brightYellow(
                    buildString {
                        append(Prompts.optional)
                        append(" Enter ")
                        append(portableCommandAliasSchema.description.lowercase().replaceAfter(".", ""))
                    }
                )
            )
            println(cyan(if (relativeFilePath != null) "Installer: $relativeFilePath" else "Example: dart"))
            portableCommandAlias = prompt(
                InstallerManifest.Installer.NestedInstallerFiles::portableCommandAlias.name
                    .replaceFirstChar { it.titlecase() }
                    .replace(Regex("([A-Z])"), " $1").trim()
            )?.trim()
            val error = isPortableCommandAliasValid(portableCommandAlias)?.also { println(brightRed(it)) }
            println()
        } while (error != null)
        return portableCommandAlias.takeIf { it?.isNotBlank() == true }
    }

    private fun isPortableCommandAliasValid(portableCommandAlias: String?): String? {
        val portableCommandAliasSchema = nestedInstallerPropertiesSchema.portableCommandAlias
        return when {
            portableCommandAlias.isNullOrBlank() -> null
            portableCommandAlias.length > portableCommandAliasSchema.maxLength -> {
                Errors.invalidLength(
                    min = portableCommandAliasSchema.minLength,
                    max = portableCommandAliasSchema.maxLength
                )
            }
            else -> null
        }
    }

    private fun isRelativeFilePathValid(relativeFilePath: String?): String? {
        val relativeFilePathSchema = installerSchema.definitions.nestedInstallerFiles.items.properties.relativeFilePath
        return when {
            relativeFilePath.isNullOrBlank() -> Errors.blankInput(
                InstallerManifest.Installer.NestedInstallerFiles::relativeFilePath.name
                    .replaceFirstChar { it.titlecase() }
                    .replace(Regex("([A-Z])"), " $1").trim()
            )
            relativeFilePath.length > relativeFilePathSchema.maxLength -> {
                Errors.invalidLength(
                    min = relativeFilePathSchema.minLength,
                    max = relativeFilePathSchema.maxLength
                )
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
        chosenZipEntries: List<ZipEntry>,
        zipFile: ZipFile
    ): InstallerManifest.Installer.NestedInstallerType {
        val smallestEntry = chosenZipEntries.minBy { it.size }
        return when (smallestEntry.name.substringAfterLast(".").lowercase()) {
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
                    /* prefix = */ smallestEntry.name,
                    /* suffix = */ InstallerManifest.Installer.NestedInstallerType.MSI.toString()
                )
                zipFile.getInputStream(smallestEntry).use { input ->
                    tempFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                if (Msi(tempFile).isWix.also { tempFile.delete() }) {
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
                ).map { it.toString() }
                println(brightGreen("${Prompts.required} Enter the nested installer type"))
                println(cyan("Options: ${exeNestedTypes.joinToString(", ")}"))
                prompt(
                    prompt = Prompts.enterChoice,
                    convert = { string ->
                        if (string.lowercase() in exeNestedTypes) {
                            ConversionResult.Valid(
                                InstallerManifest.Installer.NestedInstallerType.valueOf(string.uppercase())
                            )
                        } else {
                            ConversionResult.Invalid(Errors.invalidEnum(validation = null, enum = exeNestedTypes))
                        }
                    }
                ) ?: InstallerManifest.Installer.NestedInstallerType.EXE
            }
            else -> {
                val nestedInstallerTypes = InstallerManifest.Installer.NestedInstallerType
                    .values().map { it.toString() }
                prompt(
                    prompt = Prompts.enterChoice,
                    convert = { string ->
                        if (string.lowercase() in nestedInstallerTypes) {
                            ConversionResult.Valid(
                                InstallerManifest.Installer.NestedInstallerType.valueOf(string.uppercase())
                            )
                        } else {
                            ConversionResult.Invalid(Errors.invalidEnum(validation = null, enum = nestedInstallerTypes))
                        }
                    }
                ) ?: InstallerManifest.Installer.NestedInstallerType.EXE
            }
        }.also { println() }
    }
}
