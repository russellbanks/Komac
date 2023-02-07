package data

import detection.PageScraper
import detection.files.Zip
import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import detection.github.GitHubDetection
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import schemas.AdditionalMetadata

@Single
class SharedManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    var publisher: String? = null
    var packageName: String? = null
    var defaultLocale: String = ""
    var updateState: VersionUpdateState? = null
    var latestVersion: String? = null
    var allVersions: List<String>? = null
    var msix: Msix? = null
    var msixBundle: MsixBundle? = null
    var msi: Msi? = null
    var zip: Zip? = null
    var gitHubDetection: GitHubDetection? = null
    var pageScraper: PageScraper? = null
    var additionalMetadata: AdditionalMetadata? = null
}
