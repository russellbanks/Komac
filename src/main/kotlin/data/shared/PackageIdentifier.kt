package data.shared

import Errors
import ExitCode
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import input.Prompts
import kotlinx.coroutines.runBlocking
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.SchemasImpl
import schemas.data.InstallerSchema
import token.TokenStore
import java.io.IOException
import kotlin.system.exitProcess

object PackageIdentifier : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var installerSchema: InstallerSchema
    private val tokenStore: TokenStore by inject()

    suspend fun Terminal.packageIdentifierPrompt(
        packageIdentifierParameter: String? = null,
        isCIEnvironment: Boolean = false
    ) {
        val schemasImpl: SchemasImpl = get()
        if (packageIdentifierParameter == null && !isCIEnvironment) {
            if (tokenStore.token == null) {
                tokenStore.promptForToken(this)
            }
            println(colors.brightGreen(identifierInfo))
            info(example)
            sharedManifestData.packageIdentifier = prompt(
                prompt = const,
                convert = { input ->
                    if (!::installerSchema.isInitialized) {
                        runBlocking {
                            schemasImpl.awaitSchema(
                                schema = Schema.Installer,
                                terminal = this@packageIdentifierPrompt
                            )
                        }
                        installerSchema = schemasImpl.installerSchema
                    }
                    isPackageIdentifierValid(input)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            ) ?: exitProcess(ExitCode.CtrlC.code)
            if (!tokenStore.isTokenValid.await()) {
                println()
                tokenStore.invalidTokenPrompt(this)
            }
            sharedManifestData.latestVersion = getLatestVersion(sharedManifestData.packageIdentifier)
            println()
        } else if (packageIdentifierParameter != null) {
            schemasImpl.awaitSchema(schema = Schema.Installer, terminal = this)
            installerSchema = schemasImpl.installerSchema
            sharedManifestData.packageIdentifier = packageIdentifierParameter
            sharedManifestData.latestVersion = getLatestVersion(
                packageIdentifier = packageIdentifierParameter,
                writeOutput = false
            )
        } else {
            throw CliktError(colors.danger("Package Identifier not inputted"), statusCode = 1)
        }
    }

    private suspend fun Terminal.getLatestVersion(packageIdentifier: String, writeOutput: Boolean = true): String? {
        return try {
            get<GitHubImpl>()
                .getMicrosoftWingetPkgs()
                ?.getDirectoryContent(Ktor.getDirectoryPath(packageIdentifier))
                ?.filter { it.name.matches(Regex(installerSchema.definitions.packageVersion.pattern)) }
                ?.filter { it.isDirectory }
                ?.also {
                    if (it.isNotEmpty() && writeOutput) info("Found $packageIdentifier in the winget-pkgs repository")
                }
                ?.map { it.name }
                ?.also { sharedManifestData.allVersions = it }
                ?.let { PackageVersion.getHighestVersion(it) }
                ?.also { if (writeOutput) info("Found latest version: $it") }
                .also { sharedManifestData.latestVersion = it }
        } catch (_: IOException) {
            sharedManifestData.updateState = VersionUpdateState.NewPackage
            null
        }
    }

    private fun isPackageIdentifierValid(identifier: String): String? {
        val packageIdentifierSchema = installerSchema.definitions.packageIdentifier
        return when {
            identifier.isBlank() -> Errors.blankInput(const)
            identifier.length > packageIdentifierSchema.maxLength -> {
                Errors.invalidLength(min = minLength, max = packageIdentifierSchema.maxLength)
            }
            !identifier.matches(Regex(packageIdentifierSchema.pattern)) -> {
                Errors.invalidRegex(Regex(packageIdentifierSchema.pattern))
            }
            else -> null
        }
    }

    private const val const = "Package Identifier"
    private const val example = "Example: Microsoft.Excel"
    private const val identifierInfo = "${Prompts.required} Enter the $const, " +
        "in the following format <Publisher shortname.Application shortname>"
    private const val minLength = 4
}
