package data.shared

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.terminal.Terminal
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import input.PromptType
import input.Prompts
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.SchemasImpl
import schemas.data.InstallerSchema
import java.io.IOException
import kotlin.system.exitProcess

object PackageIdentifier : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var installerSchema: InstallerSchema

    suspend fun Terminal.packageIdentifierPrompt(packageIdentifier: String? = null) {
        val schemasImpl: SchemasImpl = get()
        if (packageIdentifier != null) {
            schemasImpl.awaitSchema(schema = Schema.Installer, terminal = this)
            installerSchema = get<SchemasImpl>().installerSchema
            sharedManifestData.packageIdentifier = packageIdentifier
            findPreviousVersions(packageIdentifier = packageIdentifier, writeOutput = false).let {
                if (it == null) {
                    println(brightRed("$packageIdentifier does not exist in winget-pkgs"))
                    exitProcess(0)
                }
            }
        } else {
            do {
                println(brightGreen(packageIdentifierInfo))
                println(cyan(packageIdentifierExample))
                val input = prompt(brightWhite(PromptType.PackageIdentifier.toString()))?.trim()
                schemasImpl.awaitSchema(schema = Schema.Installer, terminal = this)
                installerSchema = get<SchemasImpl>().installerSchema
                val error = isPackageIdentifierValid(input).also {
                    if (it == null) {
                        if (input != null) {
                            sharedManifestData.packageIdentifier = input
                            findPreviousVersions(input)
                        }
                    } else {
                        println(brightRed(it))
                    }
                }
                println()
            } while (error != null)
        }
    }

    private suspend fun Terminal.findPreviousVersions(packageIdentifier: String, writeOutput: Boolean = true): String? {
        return try {
            get<GitHubImpl>()
                .getMicrosoftWingetPkgs()
                ?.getDirectoryContent(Ktor.getDirectoryPath(packageIdentifier))
                ?.filter { it.name.matches(Regex(installerSchema.definitions.packageVersion.pattern)) }
                ?.filter { it.isDirectory }
                ?.also {
                    if (it.isNotEmpty() && writeOutput) {
                        println(cyan("Found $packageIdentifier in the winget-pkgs repository"))
                    }
                }
                ?.map { it.name }
                ?.let { PackageVersion.getHighestVersion(it) }
                ?.also { if (writeOutput) println(cyan("Found latest version: $it")) }
                .also {
                    sharedManifestData.latestVersion = it
                    return it
                }
        } catch (_: IOException) {
            sharedManifestData.updateState = VersionUpdateState.NewPackage
            null
        }
    }

    private fun isPackageIdentifierValid(identifier: String?): String? {
        val packageIdentifierSchema = installerSchema.definitions.packageIdentifier
        return when {
            identifier.isNullOrBlank() -> Errors.blankInput(PromptType.PackageIdentifier)
            identifier.length > packageIdentifierSchema.maxLength -> {
                Errors.invalidLength(
                    min = packageIdentifierMinLength,
                    max = packageIdentifierSchema.maxLength
                )
            }
            !identifier.matches(Regex(packageIdentifierSchema.pattern)) -> {
                Errors.invalidRegex(Regex(packageIdentifierSchema.pattern))
            }
            else -> null
        }
    }

    private const val packageIdentifierExample = "Example: Microsoft.Excel"
    private const val packageIdentifierInfo = "${Prompts.required} Enter the Package Identifier, " +
        "in the following format <Publisher shortname.Application shortname>"
    private const val packageIdentifierMinLength = 4
}
