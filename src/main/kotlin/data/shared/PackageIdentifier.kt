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
import network.HttpUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import token.TokenStore
import java.io.IOException
import kotlin.system.exitProcess

object PackageIdentifier : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val tokenStore: TokenStore by inject()

    suspend fun Terminal.packageIdentifierPrompt(
        packageIdentifierParameter: String? = null,
        isCIEnvironment: Boolean = false
    ) {
        get<SchemasImpl>()
        if (packageIdentifierParameter == null && !isCIEnvironment) {
            if (tokenStore.token == null) {
                tokenStore.promptForToken(this)
            }
            println(colors.brightGreen(identifierInfo))
            info(example)
            sharedManifestData.packageIdentifier = prompt(
                prompt = const,
                convert = { input ->
                    getPackageIdentifierError(input)
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
            sharedManifestData.packageIdentifier = packageIdentifierParameter
            sharedManifestData.latestVersion = getLatestVersion(
                packageIdentifier = packageIdentifierParameter,
                writeOutput = false
            )
        } else {
            throw CliktError(colors.danger("${Errors.error} Package Identifier not provided"), statusCode = 1)
        }
    }

    private suspend fun Terminal.getLatestVersion(packageIdentifier: String, writeOutput: Boolean = true): String? {
        return try {
            get<GitHubImpl>()
                .getMicrosoftWingetPkgs()
                ?.getDirectoryContent(HttpUtils.getDirectoryPath(packageIdentifier))
                ?.filter { it.name.matches(regex) }
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

    fun getPackageIdentifierError(identifier: String): String? {
        return when {
            identifier.isBlank() -> Errors.blankInput(const)
            identifier.length > maxLength -> Errors.invalidLength(min = minLength, max = maxLength)
            !identifier.matches(regex) -> Errors.invalidRegex(regex)
            else -> null
        }
    }

    private const val const = "Package Identifier"
    private const val example = "Example: Microsoft.Excel"
    private const val identifierInfo = "${Prompts.required} Enter the $const, " +
        "in the following format <Publisher shortname.Application shortname>"
    const val maxLength = 128
    const val minLength = 4
    private const val pattern = "^[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}(\\.[^.\\s\\\\/:*?\"<>|\\x01-\\x1f]{1,32}){1,7}$"
    private val regex = Regex(pattern)
}
