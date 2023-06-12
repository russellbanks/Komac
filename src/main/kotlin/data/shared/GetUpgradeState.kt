package data.shared

import github.GitHubImpl
import data.VersionUpdateState
import utils.versionStringComparator

fun getUpdateState(
    packageIdentifier: String,
    packageVersion: String,
    latestVersion: String?
): VersionUpdateState {
    return when {
        latestVersion == null -> VersionUpdateState.NewPackage
        GitHubImpl.versionExists(packageIdentifier, packageVersion) -> VersionUpdateState.UpdateVersion
        packageVersion == maxOf(packageVersion, latestVersion, versionStringComparator) ->
            VersionUpdateState.NewVersion
        else -> VersionUpdateState.AddVersion
    }
}
