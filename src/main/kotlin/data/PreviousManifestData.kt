package data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import network.HttpUtils
import org.kohsuke.github.GHContent
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.VersionManifest

@Single
class PreviousManifestData : KoinComponent {
    var sharedManifestData: SharedManifestData = get()
    val scope = CoroutineScope(Dispatchers.IO)
    private val repository = scope.async { get<GitHubImpl>().getMicrosoftWingetPkgs() }
    private val directoryPath: Deferred<MutableList<GHContent>?> = scope.async {
        sharedManifestData.latestVersion?.let {
            repository.await()
                ?.getDirectoryContent("${HttpUtils.getDirectoryPath(sharedManifestData.packageIdentifier)}/$it")
        }
    }
    var remoteInstallerData: Deferred<InstallerManifest?> = scope.async {
        directoryPath.await()?.let { nonNullDirectoryPath ->
            repository.await()?.getFileContent(
                nonNullDirectoryPath.first { it.name == "${sharedManifestData.packageIdentifier}.installer.yaml" }.path
            )?.read()?.use {
                EncodeConfig.yamlDefault.decodeFromStream(InstallerManifest.serializer(), it)
            }
        }
    }
    var remoteDefaultLocaleData: Deferred<DefaultLocaleManifest?> = scope.async {
        directoryPath.await()?.let { nonNullDirectoryPath ->
            repository.await()?.getFileContent(
                nonNullDirectoryPath.first {
                    it.name == "${sharedManifestData.packageIdentifier}.locale.${remoteVersionData.await()?.defaultLocale}.yaml"
                }.path
            )?.read()?.use {
                EncodeConfig.yamlDefault.decodeFromStream(DefaultLocaleManifest.serializer(), it)
            }
        }
    }
    var remoteLocaleData: Deferred<List<LocaleManifest>?> = scope.async {
        directoryPath.await()
            ?.filter {
                it.name.matches(Regex("${Regex.escape(sharedManifestData.packageIdentifier)}.locale\\..*\\.yaml"))
            }
            ?.filterNot { ghContent ->
                remoteVersionData.await()?.defaultLocale?.let { ghContent.name.contains(it) } == true
            }
            ?.mapNotNull { ghContent ->
                repository.await()?.getFileContent(ghContent.path)
                    ?.read()
                    ?.use {
                        EncodeConfig.yamlDefault.decodeFromStream(LocaleManifest.serializer(), it)
                    }
            }
    }
    var remoteVersionData: Deferred<VersionManifest?> = scope.async {
        directoryPath.await()?.let { nonNullDirectoryPath ->
            repository.await()?.getFileContent(
                nonNullDirectoryPath.first { it.name == "${sharedManifestData.packageIdentifier}.yaml" }.path
            )?.read()?.use { EncodeConfig.yamlDefault.decodeFromStream(VersionManifest.serializer(), it) }
        }
    }
}
