package data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.VersionManifest
import utils.GitHubUtils

class PreviousManifestData(packageIdentifier: String, latestVersion: String?, microsoftWinGetPkgs: GHRepository?) {
    private val scope = CoroutineScope(Dispatchers.IO)
    private val directoryPath: MutableList<GHContent>? = latestVersion?.let {
        microsoftWinGetPkgs?.getDirectoryContent("${GitHubUtils.getPackagePath(packageIdentifier)}/$it")
    }
    var previousVersionData: VersionManifest? = directoryPath?.let { nonNullDirectoryPath ->
        microsoftWinGetPkgs
            ?.getFileContent(nonNullDirectoryPath.first { it.name == "$packageIdentifier.yaml" }.path)
            ?.read()
            ?.use { EncodeConfig.yamlDefault.decodeFromStream(VersionManifest.serializer(), it) }
    }
    var remoteInstallerData: Deferred<InstallerManifest?> = scope.async {
        directoryPath?.let { nonNullDirectoryPath ->
            microsoftWinGetPkgs?.getFileContent(
                nonNullDirectoryPath.first { it.name == GitHubUtils.getInstallerManifestName(packageIdentifier) }.path
            )?.read()?.use {
                EncodeConfig.yamlDefault.decodeFromStream(InstallerManifest.serializer(), it)
            }
        }
    }
    var remoteDefaultLocaleData: Deferred<DefaultLocaleManifest?> = scope.async {
        directoryPath?.let { nonNullDirectoryPath ->
            microsoftWinGetPkgs?.getFileContent(
                nonNullDirectoryPath.first {
                    it.name == GitHubUtils.getDefaultLocaleManifestName(
                        identifier = packageIdentifier,
                        previousDefaultLocale = previousVersionData?.defaultLocale
                    )
                }.path
            )?.read()?.use { EncodeConfig.yamlDefault.decodeFromStream(DefaultLocaleManifest.serializer(), it) }
        }
    }
    var remoteLocaleData: Deferred<List<LocaleManifest>?> = scope.async {
        directoryPath
            ?.filter { it.name matches "${Regex.escape(packageIdentifier)}.locale\\..*\\.yaml".toRegex() }
            ?.filterNot { ghContent -> previousVersionData?.defaultLocale?.let(ghContent.name::contains) == true }
            ?.mapNotNull { ghContent ->
                microsoftWinGetPkgs?.getFileContent(ghContent.path)
                    ?.read()
                    ?.use { EncodeConfig.yamlDefault.decodeFromStream(LocaleManifest.serializer(), it) }
            }
    }
}
