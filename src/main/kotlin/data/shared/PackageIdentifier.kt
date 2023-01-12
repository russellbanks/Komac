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
import schemas.InstallerSchema
import schemas.Schema
import schemas.SchemasImpl
import java.io.IOException

object PackageIdentifier : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var installerSchema: InstallerSchema

    suspend fun Terminal.packageIdentifierPrompt() {
        val schemasImpl: SchemasImpl by inject()
        do {
            println(brightGreen(packageIdentifierInfo))
            println(cyan(packageIdentifierExample))
            val input = prompt(brightWhite(PromptType.PackageIdentifier.toString()))?.trim()
            schemasImpl.awaitSchema(Schema.Installer)
            installerSchema = get<SchemasImpl>().installerSchema
            val error = isPackageIdentifierValid(input).also {
                if (it == null) {
                    if (input != null) {
                        sharedManifestData.packageIdentifier = input
                        findPreviousVersions()
                    }
                } else {
                    println(brightRed(it))
                }
            }
            println()
        } while (error != null)
    }

    private fun findPreviousVersions() {
        try {
            val githubImpl = get<GitHubImpl>()
            sharedManifestData.latestVersion = githubImpl
                .getMicrosoftWingetPkgs()
                ?.getDirectoryContent(Ktor.getDirectoryPath(sharedManifestData.packageIdentifier))
                ?.filter {
                    it.name.matches(
                        Regex(get<SchemasImpl>().installerSchema.definitions.packageIdentifier.pattern)
                    )
                }
                ?.filter { it.isDirectory }
                ?.also {
                    if (it.isNotEmpty()) {
                        println(cyan("Found ${sharedManifestData.packageIdentifier} in the winget-pkgs repository"))
                    }
                }
                ?.map { it.name }
                ?.let { PackageVersion.getHighestVersion(it) }
                ?.also {
                    sharedManifestData.latestVersion = it
                    println(cyan("Found latest version: $it"))
                }.toString()
        } catch (_: IOException) {
            sharedManifestData.updateState = VersionUpdateState.NewPackage
        }
    }

    fun isPackageIdentifierValid(identifier: String?): String? {
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
