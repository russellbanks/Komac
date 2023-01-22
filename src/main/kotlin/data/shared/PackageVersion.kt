package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.GitHubImpl
import data.SharedManifestData
import data.VersionUpdateState
import input.Prompts
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import kotlin.random.Random
import kotlin.system.exitProcess

object PackageVersion : KoinComponent {
    private val githubImpl: GitHubImpl by inject()

    suspend fun Terminal.packageVersionPrompt(packageVersion: String? = null) {
        val sharedManifestData: SharedManifestData by inject()
        sharedManifestData.packageVersion = if (packageVersion != null) {
            packageVersion
        } else {
            println(colors.brightGreen(versionInfo))
            info(example)
            prompt(
                prompt = colors.brightWhite(const),
                convert = { input ->
                    isPackageVersionValid(input)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
        }
        setUpgradeState(sharedManifestData)
    }

    private suspend fun setUpgradeState(sharedManifestData: SharedManifestData) {
        if (sharedManifestData.updateState == VersionUpdateState.NewPackage) return
        val packageExistsInRepo = checkIfPackageExistsInRepo(sharedManifestData)
        if (packageExistsInRepo) {
            sharedManifestData.updateState = VersionUpdateState.UpdateVersion
        } else {
            setUpdateStateBasedOnPackageVersion(sharedManifestData)
        }
    }

    private suspend fun checkIfPackageExistsInRepo(sharedManifestData: SharedManifestData): Boolean {
        val packageNames = githubImpl.getMicrosoftWingetPkgs()
            ?.getDirectoryContent(Ktor.getDirectoryPath(sharedManifestData.packageIdentifier))
            ?.map { it.name }
        return packageNames?.contains(sharedManifestData.packageVersion) ?: false
    }

    private fun setUpdateStateBasedOnPackageVersion(sharedManifestData: SharedManifestData) {
        val versionsToCompare = listOf(sharedManifestData.packageVersion, sharedManifestData.latestVersion)
        val highestVersion = getHighestVersion(versionsToCompare.filterNotNull())
        if (sharedManifestData.packageVersion == highestVersion) {
            sharedManifestData.updateState = VersionUpdateState.NewVersion
        } else {
            sharedManifestData.updateState = VersionUpdateState.AddVersion
        }
    }

    private fun isPackageVersionValid(version: String): String? {
        val packageVersionSchema = get<SchemasImpl>().installerSchema.definitions.packageVersion
        return when {
            version.isBlank() -> Errors.blankInput(const)
            version.length > packageVersionSchema.maxLength -> {
                Errors.invalidLength(max = packageVersionSchema.maxLength)
            }
            !version.matches(Regex(packageVersionSchema.pattern)) -> {
                Errors.invalidRegex(Regex(packageVersionSchema.pattern))
            }
            else -> null
        }
    }

    fun getHighestVersion(versions: List<String>): String {
        val toNatural: (String) -> String = {
            Regex("\\d+").replace(it) { matchResult ->
                matchResult.value.padStart(20)
            }
        }
        data class VersionPart(val value: Int, val supplement: String, val original: String)

        fun parseVersionPart(part: String): VersionPart {
            val value = part.takeWhile { it.isDigit() }.toIntOrNull() ?: 0
            val supplement = part.dropWhile { it.isDigit() }
            return VersionPart(value, supplement, part)
        }

        fun compareVersionParts(left: VersionPart, right: VersionPart): Int {
            return when {
                left.value != right.value -> left.value.compareTo(right.value)
                left.supplement.isEmpty() && right.supplement.isEmpty() -> 0
                left.supplement.isEmpty() -> 1
                right.supplement.isEmpty() -> -1
                else -> left.supplement.compareTo(right.supplement)
            }
        }

        fun compareVersions(left: List<VersionPart>, right: List<VersionPart>): Int {
            return left.zip(right).map { compareVersionParts(it.first, it.second) }.firstOrNull { it != 0 } ?: 0
        }

        return versions.asSequence()
            .sortedWith(compareBy(toNatural))
            .map { version ->
                version.split(".").map { versionPart ->
                    parseVersionPart(versionPart)
                }
            }.sortedWith { left, right ->
                compareVersions(left, right)
            }.last().joinToString(".") { it.original }
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
}
