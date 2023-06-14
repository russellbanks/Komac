package data

import github.GitHubUtils
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

class PreviousManifestData(
    private val packageIdentifier: String,
    latestVersion: String?,
    private val microsoftWinGetPkgs: GHRepository
) {
    private val scope = CoroutineScope(Dispatchers.IO)
    private var directoryPath: List<GHContent>? = latestVersion?.let {
        microsoftWinGetPkgs.getDirectoryContent("${GitHubUtils.getPackagePath(packageIdentifier)}/$it")
    }

    private lateinit var previousVersionDataDeferred: Deferred<VersionManifest?>
    var installerManifest: Deferred<InstallerManifest?> = scope.async {
        directoryPath?.let { nonNullDirectoryPath ->
            microsoftWinGetPkgs.getFileContent(
                nonNullDirectoryPath.first { it.name == InstallerManifest.getFileName(packageIdentifier) }.path
            )?.read()?.use {
                EncodeConfig.yamlDefault.decodeFromStream(InstallerManifest.serializer(), it)
            }
        }
    }
    private lateinit var remoteDefaultLocaleDataDeferred: Deferred<DefaultLocaleManifest?>
    private lateinit var remoteLocaleDataDeferred: Deferred<List<LocaleManifest>?>

    init {
        scope.launch {
            previousVersionDataDeferred = scope.async {
                directoryPath?.let { nonNullDirectoryPath ->
                    microsoftWinGetPkgs
                        .getFileContent(nonNullDirectoryPath.first { it.name == "$packageIdentifier.yaml" }.path)
                        ?.read()
                        ?.use { EncodeConfig.yamlDefault.decodeFromStream(VersionManifest.serializer(), it) }
                }
            }
            remoteDefaultLocaleDataDeferred = scope.async {
                directoryPath?.let { nonNullDirectoryPath ->
                    microsoftWinGetPkgs.getFileContent(
                        nonNullDirectoryPath.first {
                            it.name == DefaultLocaleManifest.getFileName(
                                identifier = packageIdentifier,
                                previousDefaultLocale = versionManifest?.defaultLocale
                            )
                        }.path
                    )?.read()?.use { EncodeConfig.yamlDefault.decodeFromStream(DefaultLocaleManifest.serializer(), it) }
                }
            }
            remoteLocaleDataDeferred = scope.async {
                directoryPath
                    ?.filter { it.name matches "${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml".toRegex() }
                    ?.filterNot { ghContent ->
                        previousVersionDataDeferred
                            .await()
                            ?.defaultLocale
                            ?.let(ghContent.name::contains) == true
                    }
                    ?.mapNotNull { ghContent ->
                        microsoftWinGetPkgs.getFileContent(ghContent.path)
                            ?.read()
                            ?.use { EncodeConfig.yamlDefault.decodeFromStream(LocaleManifest.serializer(), it) }
                    }
            }
        }
    }

    val versionManifest by lazy { runBlocking { previousVersionDataDeferred.await() } }
    val defaultLocaleManifest by lazy { runBlocking { remoteDefaultLocaleDataDeferred.await() } }
    val localeManifests by lazy { runBlocking { remoteLocaleDataDeferred.await() } }
}
