package schemas

import data.InstallerManifestData
import data.SharedManifestData
import data.YamlConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import java.io.File
import java.io.FileWriter
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

object ManifestBuilder : KoinComponent {
    val sharedManifestData: SharedManifestData by inject()
    val installerManifestData: InstallerManifestData by inject()
    val installerManifestName = "${sharedManifestData.packageIdentifier}.installer.yaml"
    val defaultLocaleManifestName
        get() = "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
    val versionManifestName = "${sharedManifestData.packageIdentifier}.version.yaml"
    private const val komacTemp = "komac-tmp"

    private val baseGitHubPath = buildString {
        append("manifests/")
        append("${sharedManifestData.packageIdentifier.first().lowercase()}/")
        append("${sharedManifestData.packageIdentifier.replace(".", "/")}/")
        append(sharedManifestData.packageVersion)
    }

    val installerManifestGitHubPath = "$baseGitHubPath/$installerManifestName"

    val defaultLocaleManifestGitHubPath = "$baseGitHubPath/$defaultLocaleManifestName"

    val versionManifestGitHubPath = "$baseGitHubPath/$versionManifestName"

    fun getLocaleManifestGitHubPath(locale: String): String {
        return "$baseGitHubPath/${sharedManifestData.packageIdentifier}.locale.$locale.yaml"
    }

    private fun buildManifestString(schemaUrl: String, block: StringBuilder.() -> Unit): String {
        return buildString {
            appendLine(Schemas.Comments.createdBy)
            appendLine(Schemas.Comments.languageServer(schemaUrl))
            appendLine()
            block()
        }
    }

    fun createTempDirectories(): Pair<Path, Path> {
        val directories: List<String> = listOf(
            komacTemp,
            sharedManifestData.packageIdentifier.first().toString().lowercase()
        ) + sharedManifestData.packageIdentifier.split(".") + listOf(
            sharedManifestData.packageVersion
        )
        val tempDirectory: Path = Paths.get(System.getProperty("java.io.tmpdir"))
        var parent: Path = tempDirectory
        directories.forEach { directory ->
            parent = parent.resolve(directory)
            if (!Files.exists(parent)) {
                Files.createDirectory(parent)
            }
        }
        return tempDirectory.resolve(komacTemp) to parent
    }

    suspend fun Path.writeManifestsToFiles(
        installerManifest: InstallerManifest?,
        defaultLocaleManifest: DefaultLocaleManifest?,
        localeManifests: List<LocaleManifest>? = null,
        versionManifest: VersionManifest?
    ) = withContext(Dispatchers.IO) {
        val installerManifestFile = resolve(installerManifestName).toFile()
        val defaultLocaleManifestFile = resolve(defaultLocaleManifestName).toFile()
        val versionManifestFile = resolve(versionManifestName).toFile()
        installerManifest?.let {
            FileWriter(installerManifestFile).use { fileWriter ->
                buildManifestString(get<SchemasImpl>().installerSchema.id) {
                    appendLine(YamlConfig.installer.encodeToString(InstallerManifest.serializer(), installerManifest))
                }.let {
                    fileWriter.write(it.replace("\n", "\r\n"))
                }
            }
        }
        defaultLocaleManifest?.let {
            FileWriter(defaultLocaleManifestFile).use { fileWriter ->
                buildManifestString(get<SchemasImpl>().defaultLocaleSchema.id) {
                    appendLine(
                        YamlConfig.other.encodeToString(DefaultLocaleManifest.serializer(), defaultLocaleManifest)
                    )
                }.let {
                    fileWriter.write(it.replace("\n", "\r\n"))
                }
            }
        }
        localeManifests?.forEach { localeManifest ->
            val localeManifestFile = resolve(
                "${sharedManifestData.packageIdentifier}.locale.${localeManifest.packageLocale}.yaml"
            ).toFile()
            FileWriter(localeManifestFile).use { fileWriter ->
                buildManifestString(get<SchemasImpl>().localeSchema.id) {
                    appendLine(YamlConfig.other.encodeToString(LocaleManifest.serializer(), localeManifest))
                }.let {
                    fileWriter.write(it.replace("\n", "\r\n"))
                }
            }
        }
        versionManifest?.let {
            FileWriter(versionManifestFile).use { fileWriter ->
                buildManifestString(get<SchemasImpl>().versionSchema.id) {
                    appendLine(YamlConfig.other.encodeToString(VersionManifest.serializer(), versionManifest))
                }.let {
                    fileWriter.write(it.replace("\n", "\r\n"))
                }
            }
        }
        return@withContext listOf<File?>(
            installerManifestFile,
            defaultLocaleManifestFile,
            versionManifestFile
        )
    }
}
