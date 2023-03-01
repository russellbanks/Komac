package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import data.GitHubImpl
import data.VersionUpdateState
import input.ExitCode
import input.Prompts
import network.HttpUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.math.BigInteger
import kotlin.random.Random
import kotlin.system.exitProcess

object PackageVersion : KoinComponent, CommandPrompt<String> {
    private val githubImpl: GitHubImpl by inject()
    val allManifestData: AllManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        println(colors.brightGreen(versionInfo))
        info(example)
        prompt(
            prompt = const,
            convert = { input ->
                getError(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
    }

    fun setUpgradeState(allManifestData: AllManifestData) = with(allManifestData) {
        when {
            updateState == VersionUpdateState.NewPackage -> Unit
            packageExists(packageIdentifier, packageVersion) -> updateState = VersionUpdateState.UpdateVersion
            else -> {
                val versionsToCompare = listOf(packageVersion, latestVersion)
                val highestVersion = versionsToCompare.filterNotNull().getHighestVersion()
                updateState = when (packageVersion) {
                    highestVersion -> VersionUpdateState.NewVersion
                    else -> VersionUpdateState.AddVersion
                }
            }
        }
    }

    private fun packageExists(packageIdentifier: String, packageVersion: String): Boolean {
        return githubImpl.getMicrosoftWinGetPkgs()
            ?.getDirectoryContent(HttpUtils.getDirectoryPath(packageIdentifier))
            ?.map { it.name }
            ?.contains(packageVersion)
            ?: false
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length > maxLength -> Errors.invalidLength(max = maxLength)
            !input.matches(regex) -> Errors.invalidRegex(regex)
            else -> null
        }
    }

    data class VersionPart(val value: BigInteger, val supplement: String, val original: String)

    /**
     * Parses a version part string into a VersionPart object.
     */
    private fun parseVersionPart(part: String): VersionPart {
        val value = part.takeWhile { it.isDigit() }.toBigIntegerOrNull() ?: BigInteger.valueOf(0)
        val supplement = part.dropWhile { it.isDigit() }
        return VersionPart(value, supplement, part)
    }

    /**
     * Compares two lists of VersionPart objects lexicographically based on their values and supplements.
     *
     * The comparison is performed element by element in lexicographic order. The first pair of elements that
     * differ determines the result of the comparison. If all elements are equal, the shorter list is considered
     * less than the longer one.
     *
     * @param other the other list to compare this list to.
     * @return a negative integer, zero, or a positive integer as this list is less than, equal to, or greater than
     * the specified list, respectively.
     */
    private fun List<VersionPart>.compareTo(other: List<VersionPart>): Int {
        val size = size.coerceAtMost(other.size)
        return take(size)
            .zip(other.take(size))
            .map { (leftPart, rightPart) -> compareValuesBy(leftPart, rightPart, { it.value }, { it.supplement }) }
            .firstOrNull { it != 0 } ?: size.compareTo(other.size)
    }
    /**
     * Returns the highest version string from a list of version strings.
     */
    fun List<String>.getHighestVersion(): String? {
        return map { version -> version.split(".").map(::parseVersionPart) }
            .maxWithOrNull { left, right -> left.compareTo(right) }
            ?.joinToString(".") { it.original }
    }

    private fun generateRandomVersion(): String {
        val major = Random.nextInt(1, 10)
        val minor = Random.nextInt(0, 100)
        val patch = Random.nextInt(0, 10)
        return "$major.$minor.$patch"
    }

    private const val const = "Package Version"
    private const val versionInfo = "${Prompts.required} Enter the version."
    private val example = "Example: ${generateRandomVersion()}"
    private const val pattern = "^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"
    val regex = Regex(pattern)
    const val maxLength = 128
}
