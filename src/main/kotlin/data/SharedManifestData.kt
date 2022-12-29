package data

import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import ktor.Ktor
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.LocaleManifest
import schemas.VersionManifest
import java.io.BufferedReader
import java.io.InputStreamReader

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

    suspend fun getPreviousManifestData() = coroutineScope {
        val githubImpl = get<GitHubImpl>()
        runCatching {
            val repository = githubImpl.getMicrosoftWingetPkgs()
            val directoryPath = repository
                ?.getDirectoryContent("${Ktor.getDirectoryPath(packageIdentifier)}/$latestVersion")
            (directoryPath?.first { it.name == "$packageIdentifier.installer.yaml" }?.path)
            remoteInstallerDataJob = launch(Dispatchers.IO) {
                repository?.getFileContent(
                    directoryPath?.first { it.name == "$packageIdentifier.installer.yaml" }?.path
                )?.read()
                    ?.let { BufferedReader(InputStreamReader(it)) }
                    ?.use { reader ->
                        var line: String?
                        buildString {
                            while (reader.readLine().also { line = it } != null) {
                                appendLine(line)
                            }
                        }.let {
                            remoteInstallerData = YamlConfig.installer.decodeFromString(
                                InstallerManifest.serializer(),
                                it
                            )
                        }
                    }
            }
            remoteVersionDataJob = launch(Dispatchers.IO) {
                repository?.getFileContent(directoryPath?.first { it.name == "$packageIdentifier.yaml" }?.path)
                    ?.read()
                    ?.let { BufferedReader(InputStreamReader(it)) }
                    ?.use { reader ->
                        var line: String?
                        buildString {
                            while (reader.readLine().also { line = it } != null) {
                                appendLine(line)
                            }
                        }.let {
                            remoteVersionData = YamlConfig.other.decodeFromString(VersionManifest.serializer(), it)
                        }
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
        repository: GHRepository?,
        directoryPath: List<GHContent>?
    ) = coroutineScope {
        remoteDefaultLocaleDataJob = launch(Dispatchers.IO, CoroutineStart.LAZY) {
            repository?.getFileContent(
                directoryPath?.first { it.name == "$packageIdentifier.locale.$defaultLocale.yaml" }?.path
            )?.read()
                ?.let { BufferedReader(InputStreamReader(it)) }
                ?.use { reader ->
                    var line: String?
                    buildString {
                        while (reader.readLine().also { line = it } != null) {
                            appendLine(line)
                        }
                    }.let {
                        remoteDefaultLocaleData = YamlConfig.other.decodeFromString(
                            DefaultLocaleManifest.serializer(),
                            it
                        )
                    }
                }
        }
    }

    private suspend fun getRemoteLocaleData(
        repository: GHRepository?,
        directoryPath: List<GHContent>?
    ) = coroutineScope {
        remoteLocaleDataJob = launch(Dispatchers.IO, CoroutineStart.LAZY) {
            directoryPath
                ?.filter { it.name.matches(Regex("${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml")) }
                ?.filterNot { it.name.contains(defaultLocale) }
                ?.forEach { file ->
                    repository?.getFileContent(file.path)
                        ?.read()
                        ?.let { BufferedReader(InputStreamReader(it)) }
                        ?.use { reader ->
                            var line: String?
                            buildString {
                                while (reader.readLine().also { line = it } != null) {
                                    appendLine(line)
                                }
                            }.let {
                                remoteLocaleData = remoteLocaleData?.plus(
                                    YamlConfig.other.decodeFromString(LocaleManifest.serializer(), it)
                                )
                            }
                        }
                }
        }
    }
}
