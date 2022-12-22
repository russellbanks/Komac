package data

import com.charleskorn.kaml.Yaml
import com.github.ajalt.mordant.rendering.TextColors.cyan
import data.shared.GitHubDirectory
import data.shared.PackageVersion.getLatestVersion
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.serialization.modules.SerializersModule
import ktor.Clients
import ktor.Ktor
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.LocalDateSerializer
import schemas.LocaleManifest
import schemas.VersionManifest
import java.time.LocalDate

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    lateinit var defaultLocale: String
    var isNewPackage = false
    var remoteInstallerData: InstallerManifest? = null
    var remoteDefaultLocaleData: DefaultLocaleManifest? = null
    var remoteLocaleData: List<LocaleManifest>? = null
    var remoteVersionData: VersionManifest? = null
    lateinit var githubDirectory: ArrayList<GitHubDirectory.GitHubDirectoryItem>
    lateinit var latestVersion: String
    lateinit var subDirectory: ArrayList<GitHubDirectory.GitHubDirectoryItem>

    suspend fun getPreviousManifestData() = coroutineScope {
        val client: HttpClient = get<Clients>().httpClient
        val contentNegotiationClient = client.config { install(ContentNegotiation) { json() } }

        val yaml = Yaml(SerializersModule { contextual(LocalDate::class, LocalDateSerializer) })

        val directoryJob: Deferred<ArrayList<GitHubDirectory.GitHubDirectoryItem>> = async(Dispatchers.IO) {
            contentNegotiationClient.get(Ktor.getDirectoryUrl(packageIdentifier)).body()
        }
        val latestVersionJob = async(Dispatchers.Default) {
            val githubDirectory = directoryJob.await()
            githubDirectory.getLatestVersion().also {
                println(cyan("Found latest version: $it"))
            }
        }

        // Wait for the latest version to be found, then download the subdirectory in parallel
        val subDirectoryJob: Deferred<ArrayList<GitHubDirectory.GitHubDirectoryItem>> = async(Dispatchers.IO) {
            val latestVersion = latestVersionJob.await()
            contentNegotiationClient.get(githubDirectory.first { it.name == latestVersion }.links.self).body()
        }
        val remoteInstallerDataJob = async(Dispatchers.IO) {
            val subDirectory = subDirectoryJob.await()
            subDirectory
                .first { it.name == "$packageIdentifier.installer.yaml" }
                .downloadUrl?.let {
                    yaml.decodeFromString(InstallerManifest.serializer(), client.get(it).body())
                }
        }
        val remoteVersionDataDeferred = async(Dispatchers.IO) {
            val subDirectory = subDirectoryJob.await()
            subDirectory
                .first { it.name == "$packageIdentifier.yaml" }.downloadUrl?.let {
                    yaml.decodeFromString(VersionManifest.serializer(), client.get(it).body())
                }
        }

        // Wait for the subdirectory to be downloaded, then download the default locale data in parallel
        val remoteDefaultLocaleDataJob = async(Dispatchers.IO) {
            val subDirectory = subDirectoryJob.await()
            val remoteVersionData = remoteVersionDataDeferred.await()
            subDirectory
                .first {
                    it.name == buildString {
                        append(packageIdentifier)
                        append(".locale.")
                        append(remoteVersionData?.defaultLocale)
                        append(".yaml")
                    }
                }.downloadUrl?.let {
                    yaml.decodeFromString(DefaultLocaleManifest.serializer(), client.get(it).body())
                }
        }

        val remoteLocaleDataJob = async(Dispatchers.IO) {
            val subDirectory = subDirectoryJob.await()
            val remoteVersionData = remoteVersionDataDeferred.await()
            subDirectory.filter { directoryItem ->
                directoryItem.name.matches(
                    Regex("${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml")
                )
            }.filterNot { directoryItem ->
                remoteVersionData?.defaultLocale?.let { defaultLocale ->
                    directoryItem.name.contains(defaultLocale)
                } == true
            }.map {
                it.downloadUrl?.let { url -> client.get(url).body<String>() }
            }.mapNotNull { rawYamlData ->
                yaml.decodeFromString(LocaleManifest.serializer(), rawYamlData ?: return@mapNotNull null)
            }
        }

        // Assign the results of the async jobs to the global variables
        githubDirectory = directoryJob.await()
        latestVersion = latestVersionJob.await()
        subDirectory = subDirectoryJob.await()
        remoteInstallerData = remoteInstallerDataJob.await()
        remoteVersionData = remoteVersionDataDeferred.await()
        remoteDefaultLocaleData = remoteDefaultLocaleDataJob.await()
        remoteLocaleData = remoteLocaleDataJob.await()

        // Close the HTTP client
        contentNegotiationClient.close()
    }
}
