package data

import com.charleskorn.kaml.Yaml
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.modules.SerializersModule
import ktor.Ktor
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.LocalDateSerializer
import schemas.LocaleManifest
import schemas.VersionManifest
import java.io.BufferedReader
import java.io.InputStreamReader
import java.time.LocalDate

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    lateinit var defaultLocale: String
    var isNewPackage = false
    var remoteInstallerData: InstallerManifest? = null
    var remoteInstallerDataJob: Job? = null
    var remoteDefaultLocaleData: DefaultLocaleManifest? = null
    var remoteDefaultLocaleDataJob: Job? = null
    var remoteLocaleData: List<LocaleManifest>? = null
    var remoteLocaleDataJob: Job? = null
    var remoteVersionData: VersionManifest? = null
    var remoteVersionDataJob: Job? = null
    lateinit var latestVersion: String
    private val yaml = Yaml(SerializersModule { contextual(LocalDate::class, LocalDateSerializer) })

    suspend fun getPreviousManifestData() = coroutineScope {
        val github = get<GitHubImpl>().github
        runCatching {
            val repository = github.getRepository("Microsoft/winget-pkgs")
            val directoryPath = repository
                .getDirectoryContent("${Ktor.getDirectoryPath(packageIdentifier)}/$latestVersion")
            (directoryPath.first { it.name == "$packageIdentifier.installer.yaml" }.path)
            remoteInstallerDataJob = launch(Dispatchers.IO) {
                repository.getFileContent(directoryPath.first { it.name == "$packageIdentifier.installer.yaml" }.path)
                    .read()
                    .let { BufferedReader(InputStreamReader(it)) }
                    .use { reader ->
                        var line: String?
                        buildString {
                            while (reader.readLine().also { line = it } != null) {
                                appendLine(line)
                            }
                        }.let { remoteInstallerData = yaml.decodeFromString(InstallerManifest.serializer(), it) }
                    }
            }
            remoteVersionDataJob = launch(Dispatchers.IO) {
                repository.getFileContent(directoryPath.first { it.name == "$packageIdentifier.yaml" }.path)
                    .read()
                    .let { BufferedReader(InputStreamReader(it)) }
                    .use { reader ->
                        var line: String?
                        buildString {
                            while (reader.readLine().also { line = it } != null) {
                                appendLine(line)
                            }
                        }.let { remoteVersionData = yaml.decodeFromString(VersionManifest.serializer(), it) }
                    }
            }.also { job ->
                job.invokeOnCompletion {
                    remoteVersionData?.defaultLocale?.let {
                        defaultLocale = it
                    }
                }
                remoteDefaultLocaleDataJob?.join()
                remoteLocaleDataJob?.join()
            }
            getRemoteDefaultLocaleData(repository, directoryPath)
            getRemoteLocaleData(repository, directoryPath)
        }
    }

    private suspend fun getRemoteDefaultLocaleData(
        repository: GHRepository,
        directoryPath: List<GHContent>
    ) = coroutineScope {
        remoteDefaultLocaleDataJob = launch(Dispatchers.IO, CoroutineStart.LAZY) {
            repository.getFileContent(
                directoryPath.first { it.name == "$packageIdentifier.locale.$defaultLocale.yaml" }.path
            ).read()
                .let { BufferedReader(InputStreamReader(it)) }
                .use { reader ->
                    var line: String?
                    buildString {
                        while (reader.readLine().also { line = it } != null) {
                            appendLine(line)
                        }
                    }.let { remoteDefaultLocaleData = yaml.decodeFromString(DefaultLocaleManifest.serializer(), it) }
                }
        }
    }

    private suspend fun getRemoteLocaleData(
        repository: GHRepository,
        directoryPath: List<GHContent>
    ) = coroutineScope {
        remoteLocaleDataJob = launch(Dispatchers.IO, CoroutineStart.LAZY) {
            directoryPath
                .filter { it.name.matches(Regex("${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml")) }
                .filterNot { it.name.contains(defaultLocale) }
                .forEach { file ->
                    repository.getFileContent(file.path)
                        .read()
                        .let { BufferedReader(InputStreamReader(it)) }
                        .use { reader ->
                            var line: String?
                            buildString {
                                while (reader.readLine().also { line = it } != null) {
                                    appendLine(line)
                                }
                            }.let {
                                remoteLocaleData =
                                    remoteLocaleData?.plus(yaml.decodeFromString(LocaleManifest.serializer(), it))
                            }
                        }
                }
        }
    }
}
