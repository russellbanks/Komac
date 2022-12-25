package data

import com.charleskorn.kaml.Yaml
import data.shared.GitHubDirectory
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.http.isSuccess
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
    lateinit var remoteInstallerData: Deferred<InstallerManifest?>
    lateinit var remoteDefaultLocaleData: Deferred<DefaultLocaleManifest?>
    lateinit var remoteLocaleData: Deferred<List<LocaleManifest>?>
    lateinit var remoteVersionData: Deferred<VersionManifest?>
    var githubDirectory: ArrayList<GitHubDirectory.GitHubDirectoryItem>? = null
    lateinit var latestVersion: String
    var subDirectory: Deferred<ArrayList<GitHubDirectory.GitHubDirectoryItem>?>? = null
    private val client: HttpClient = get<Clients>().httpClient
    private val contentNegotiationClient = client.config { install(ContentNegotiation) { json() } }

    suspend fun doesPackageAlreadyExist(identifier: String = packageIdentifier): Boolean {
        val directoryResponse = contentNegotiationClient.get(Ktor.getDirectoryUrl(identifier))
        if (!directoryResponse.status.isSuccess()) return false
        githubDirectory = directoryResponse.body()
        return githubDirectory?.filterNot { it.name == ".validation" }?.isNotEmpty() == true
    }

    suspend fun getPreviousManifestData() = coroutineScope {
        val yaml = Yaml(SerializersModule { contextual(LocalDate::class, LocalDateSerializer) })

        // Wait for the latest version to be found, then download the subdirectory in parallel
        subDirectory = async(Dispatchers.IO) {
            githubDirectory?.first { it.name == latestVersion }?.links?.self?.let {
                contentNegotiationClient.get(it).body()
            }
        }
        remoteInstallerData = async(Dispatchers.IO) {
            val subDirectory = subDirectory?.await()
            subDirectory
                ?.first { it.name == "$packageIdentifier.installer.yaml" }
                ?.downloadUrl?.let {
                    yaml.decodeFromString(InstallerManifest.serializer(), client.get(it).body())
                }
        }
        remoteVersionData = async(Dispatchers.IO) {
            val subDirectory = subDirectory?.await()
            subDirectory
                ?.first { it.name == "$packageIdentifier.yaml" }?.downloadUrl?.let {
                    yaml.decodeFromString(VersionManifest.serializer(), client.get(it).body())
                }
        }

        remoteDefaultLocaleData = async(Dispatchers.IO) {
            val subDirectory = subDirectory?.await()
            val remoteVersionData = remoteVersionData.await()
            subDirectory
                ?.first {
                    it.name == buildString {
                        append(packageIdentifier)
                        append(".locale.")
                        append(remoteVersionData?.defaultLocale)
                        append(".yaml")
                    }
                }?.downloadUrl?.let {
                    yaml.decodeFromString(DefaultLocaleManifest.serializer(), client.get(it).body())
                }
        }

        remoteLocaleData = async(Dispatchers.IO) {
            val subDirectory = subDirectory?.await()
            val remoteVersionData = remoteVersionData.await()
            subDirectory?.filter { directoryItem ->
                directoryItem.name.matches(
                    Regex("${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml")
                )
            }?.filterNot { directoryItem ->
                remoteVersionData?.defaultLocale?.let { defaultLocale ->
                    directoryItem.name.contains(defaultLocale)
                } == true
            }?.map {
                it.downloadUrl?.let { url -> client.get(url).body<String>() }
            }?.mapNotNull { rawYamlData ->
                yaml.decodeFromString(LocaleManifest.serializer(), rawYamlData ?: return@mapNotNull null)
            }
        }

        contentNegotiationClient.close()
    }
}
