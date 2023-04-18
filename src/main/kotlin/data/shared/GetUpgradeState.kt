package data.shared

import data.GitHubImpl
import data.VersionUpdateState
import extensions.versionStringComparator

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
