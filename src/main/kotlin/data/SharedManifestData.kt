package data

import detection.GitHubDetection
import detection.files.msi.Msi
import detection.files.Msix
import detection.files.MsixBundle
import detection.files.Zip
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    var publisher: String? = null
    var packageName: String? = null
    var defaultLocale: String = ""
    var updateState: VersionUpdateState? = null
    var latestVersion: String? = null
    var msix: Msix? = null
    var msixBundle: MsixBundle? = null
    var msi: Msi? = null
    var zip: Zip? = null
    var gitHubDetection: GitHubDetection? = null
}
