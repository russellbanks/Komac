package data.shared

import data.GitHubImpl
import data.VersionUpdateState
import data.shared.PackageVersion.getHighestVersion

fun getUpdateState(
    packageIdentifier: String,
    packageVersion: String,
    latestVersion: String?,
    gitHubImpl: GitHubImpl
): VersionUpdateState {
    return when {
        latestVersion == null -> VersionUpdateState.NewPackage
        gitHubImpl.versionExists(packageIdentifier, packageVersion) -> VersionUpdateState.UpdateVersion
        packageVersion == listOf(packageVersion, latestVersion).getHighestVersion() -> VersionUpdateState.NewVersion
        else -> VersionUpdateState.AddVersion
    }
}
