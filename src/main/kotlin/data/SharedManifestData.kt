package data

import data.msi.Msi
import data.msix.Msix
import data.msix.MsixBundle
import data.zip.Zip
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    var packageName: String? = null
    var defaultLocale: String = ""
    var updateState: VersionUpdateState? = null
    var latestVersion: String? = null
    var msix: Msix? = null
    var msixBundle: MsixBundle? = null
    var msi: Msi? = null
    var zip: Zip? = null
    var fileExtension: String? = null
}
