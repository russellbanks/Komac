package data.shared

import data.GitHubImpl
import data.VersionUpdateState
import extensions.versionStringComparator

fun getUpdateState(
    packageIdentifier: String,
    packageVersion: String,
    latestVersion: String?,
    gitHubImpl: GitHubImpl
): VersionUpdateState {
    return when {
        latestVersion == null -> VersionUpdateState.NewPackage
        gitHubImpl.versionExists(packageIdentifier, packageVersion) -> VersionUpdateState.UpdateVersion
        packageVersion == maxOf(packageVersion, latestVersion, versionStringComparator) ->
            VersionUpdateState.NewVersion

        else -> VersionUpdateState.AddVersion
    }
}
