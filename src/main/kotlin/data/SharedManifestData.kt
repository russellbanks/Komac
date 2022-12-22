package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.VersionManifest

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    lateinit var defaultLocale: String
    var isNewPackage = false
    var remoteInstallerData: InstallerManifest? = null
    var remoteDefaultLocaleData: DefaultLocaleManifest? = null
    var remoteVersionData: VersionManifest? = null
}
