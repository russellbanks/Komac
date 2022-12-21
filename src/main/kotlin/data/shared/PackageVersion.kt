package data.shared

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.SharedManifestData
import input.PromptType
import input.Prompts
import io.ktor.client.call.body
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import ktor.Clients
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object PackageVersion : KoinComponent {
    suspend fun Terminal.packageVersionPrompt() {
        val sharedManifestData: SharedManifestData by inject()
        do {
            println(brightGreen(packageVersionInfo))
            val input = prompt(brightWhite(PromptType.PackageVersion.toString()))?.trim()
            val (packageVersionValid, error) = isPackageVersionValid(input)
            error?.let { println(red(it)) }
            if (packageVersionValid == Validation.Success && input != null) {
                sharedManifestData.packageVersion = input
                if (!sharedManifestData.isNewPackage) {
                    println(cyan("Found latest version: ${getLatestVersion(sharedManifestData)}"))
                }
            }
            println()
        } while (packageVersionValid != Validation.Success)
    }

    fun isPackageVersionValid(
        version: String?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageVersionSchema = installerSchema.definitions.packageVersion
        return when {
            version.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            version.length > packageVersionSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = packageVersionSchema.maxLength)
            }
            !version.matches(Regex(packageVersionSchema.pattern)) -> {
                Validation.InvalidPattern to Errors.invalidRegex(Regex(packageVersionSchema.pattern))
            }
            else -> Validation.Success to null
        }
    }

    private suspend fun getLatestVersion(
        sharedManifestData: SharedManifestData,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): String {
        get<Clients>().httpClient.config {
            install(ContentNegotiation) {
                json()
            }
        }.use { client ->
            val response = client.get(Ktor.getDirectoryUrl(sharedManifestData.packageIdentifier))
            val githubDirectory: ArrayList<GitHubDirectory.GitHubDirectoryItem> = response.body()
            val versionsList = githubDirectory.filter {
                it.name.matches(Regex(installerSchema.definitions.packageVersion.pattern))
            }.map { it.name }
            return getHighestVersion(versionsList)
        }
    }

    fun getHighestVersion(versions: List<String>): String {
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

        return versions.map { version ->
            version.split(".").map { versionPart ->
                parseVersionPart(versionPart)
            }
        }.sortedWith { left, right ->
            compareVersions(left, right)
        }.last().joinToString(".") { it.original }
    }

    private const val packageVersionInfo = "${Prompts.required} Enter the version. For example: 1.33.7"
}
