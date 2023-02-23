package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import input.ExitCode
import input.Prompts
import network.HttpUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import java.io.IOException
import kotlin.system.exitProcess

object PackageIdentifier : KoinComponent, CommandPrompt<String> {
    private val sharedManifestData: SharedManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        println(colors.brightGreen(identifierInfo))
        info(example)
        return prompt(
            prompt = const,
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    fun Terminal.getLatestVersion(packageIdentifier: String, writeOutput: Boolean = true): String? {
        return try {
            get<GitHubImpl>().getMicrosoftWinGetPkgs()
                ?.getDirectoryContent(HttpUtils.getDirectoryPath(packageIdentifier))
                ?.filter { it.name.matches(PackageVersion.regex) }
                ?.filter { it.isDirectory }
                ?.filterNot { ghContent -> ghContent.name.all { it.isLetter() } }
                ?.also {
                    if (it.isEmpty()) {
                        sharedManifestData.updateState = VersionUpdateState.NewPackage
                        return null
                    } else {
                        if (writeOutput) {
                            info("Found $packageIdentifier in the winget-pkgs repository")
                        }
                    }
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

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length > maxLength -> Errors.invalidLength(min = minLength, max = maxLength)
            !input.matches(regex) -> Errors.invalidRegex(regex)
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
