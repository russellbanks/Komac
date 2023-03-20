package data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.VersionManifest
import utils.GitHubUtils

class PreviousManifestData(packageIdentifier: String, latestVersion: String?, microsoftWinGetPkgs: GHRepository) {
    private val scope = CoroutineScope(Dispatchers.IO)
    private var directoryPath: List<GHContent>? = null

    private lateinit var previousVersionDataDeferred: Deferred<VersionManifest?>
    private lateinit var remoteInstallerDataDeferred: Deferred<InstallerManifest?>
    private lateinit var remoteDefaultLocaleDataDeferred: Deferred<DefaultLocaleManifest?>
    private lateinit var remoteLocaleDataDeferred: Deferred<List<LocaleManifest>?>

    init {
        scope.launch {
            directoryPath = latestVersion?.let {
                microsoftWinGetPkgs.getDirectoryContent("${GitHubUtils.getPackagePath(packageIdentifier)}/$it")
            }
            previousVersionDataDeferred = scope.async {
                directoryPath?.let { nonNullDirectoryPath ->
                    microsoftWinGetPkgs
                        .getFileContent(nonNullDirectoryPath.first { it.name == "$packageIdentifier.yaml" }.path)
                        ?.read()
                        ?.use { EncodeConfig.yamlDefault.decodeFromStream(VersionManifest.serializer(), it) }
                }
            }
            remoteInstallerDataDeferred = scope.async {
                directoryPath?.let { nonNullDirectoryPath ->
                    microsoftWinGetPkgs.getFileContent(
                        nonNullDirectoryPath.first { it.name == GitHubUtils.getInstallerManifestName(packageIdentifier) }.path
                    )?.read()?.use {
                        EncodeConfig.yamlDefault.decodeFromStream(InstallerManifest.serializer(), it)
                    }
                }
            }
            remoteDefaultLocaleDataDeferred = scope.async {
                directoryPath?.let { nonNullDirectoryPath ->
                    microsoftWinGetPkgs.getFileContent(
                        nonNullDirectoryPath.first {
                            it.name == GitHubUtils.getDefaultLocaleManifestName(
                                identifier = packageIdentifier,
                                previousDefaultLocale = previousVersionData?.defaultLocale
                            )
                        }.path
                    )?.read()?.use { EncodeConfig.yamlDefault.decodeFromStream(DefaultLocaleManifest.serializer(), it) }
                }
            }
            remoteLocaleDataDeferred = scope.async {
                directoryPath
                    ?.filter { it.name matches "${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml".toRegex() }
                    ?.filterNot { ghContent -> previousVersionDataDeferred.await()?.defaultLocale?.let(ghContent.name::contains) == true }
                    ?.mapNotNull { ghContent ->
                        microsoftWinGetPkgs.getFileContent(ghContent.path)
                            ?.read()
                            ?.use { EncodeConfig.yamlDefault.decodeFromStream(LocaleManifest.serializer(), it) }
                    }
            }
        }
    }

    val previousVersionData by lazy { runBlocking { previousVersionDataDeferred.await() } }
    val remoteInstallerData by lazy { runBlocking { remoteInstallerDataDeferred.await() } }
    val remoteDefaultLocaleData by lazy { runBlocking { remoteDefaultLocaleDataDeferred.await() } }
    val remoteLocaleData by lazy { runBlocking { remoteLocaleDataDeferred.await() } }
}
