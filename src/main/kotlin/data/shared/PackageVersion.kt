package data.shared

import Errors
import Validation
import com.charleskorn.kaml.Yaml
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.SharedManifestData
import input.PromptType
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import ktor.Clients
import ktor.Ktor
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl
import schemas.VersionManifest

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
                    val client: HttpClient = get<Clients>().httpClient
                    val githubDirectory = client.config {
                        install(ContentNegotiation) {
                            json()
                        }
                    }.use {
                        it.get(Ktor.getDirectoryUrl(sharedManifestData.packageIdentifier))
                            .body<ArrayList<GitHubDirectory.GitHubDirectoryItem>>()
                    }
                    val latestVersion = githubDirectory.getLatestVersion()
                    println(cyan("Found latest version: $latestVersion"))
                    val json = Json { ignoreUnknownKeys = true }
                    val subDirectory: ArrayList<GitHubDirectory.GitHubDirectoryItem> = json.decodeFromString(
                        client.get(githubDirectory.first { it.name == latestVersion }.links.self).body()
                    )
                    sharedManifestData.remoteInstallerData = subDirectory
                        .first { it.name == "${sharedManifestData.packageIdentifier}.installer.yaml" }.downloadUrl
                        ?.let { Yaml.default.decodeFromString(InstallerManifest.serializer(), client.get(it).body()) }
                    sharedManifestData.remoteVersionData = subDirectory
                        .first { it.name == "${sharedManifestData.packageIdentifier}.yaml" }.downloadUrl
                        ?.let { Yaml.default.decodeFromString(VersionManifest.serializer(), client.get(it).body()) }
                    sharedManifestData.remoteDefaultLocaleData = subDirectory
                        .first {
                            it.name == buildString {
                                append(sharedManifestData.packageIdentifier)
                                append(".locale.")
                                append(sharedManifestData.remoteVersionData?.defaultLocale)
                                append(".yaml")
                            }
                        }.downloadUrl
                        ?.let {
                            Yaml.default.decodeFromString(DefaultLocaleManifest.serializer(), client.get(it).body())
                        }
                    /* subDirectory.filter {
                        it.name.matches(
                            Regex("${Regex.escape(sharedManifestData.packageIdentifier)}.locale\\..*\\.yaml")
                        )
                    } */
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

    private fun ArrayList<GitHubDirectory.GitHubDirectoryItem>.getLatestVersion(
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): String {
        filter { it.name.matches(Regex(installerSchema.definitions.packageVersion.pattern)) }
            .map { it.name }
            .also { return getHighestVersion(it) }
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
