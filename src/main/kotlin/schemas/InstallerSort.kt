package schemas

import schemas.manifest.InstallerManifest

val installerSorter = compareBy(
    InstallerManifest.Installer::installerLocale,
    InstallerManifest.Installer::architecture,
    InstallerManifest.Installer::installerType,
    InstallerManifest.Installer::scope
)