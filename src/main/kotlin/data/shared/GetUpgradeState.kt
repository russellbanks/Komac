package data.shared

import data.GitHubImpl
import data.VersionUpdateState
import data.shared.PackageVersion.getHighestVersion

fun getUpdateState(
    updateState: VersionUpdateState,
    packageIdentifier: String,
    packageVersion: String,
    latestVersion: String?,
    gitHubImpl: GitHubImpl
): VersionUpdateState {
    return when {
        updateState == VersionUpdateState.NewPackage -> VersionUpdateState.NewPackage
        gitHubImpl.versionExists(packageIdentifier, packageVersion) -> VersionUpdateState.UpdateVersion
        else -> {
            val versionsToCompare = listOf(packageVersion, latestVersion)
            val highestVersion = versionsToCompare.filterNotNull().getHighestVersion()
            if (packageVersion == highestVersion) VersionUpdateState.NewVersion else VersionUpdateState.AddVersion
        }
    }
}
