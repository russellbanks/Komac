package data

import detection.PageScraper
import detection.files.Zip
import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import detection.github.GitHubDetection
import io.ktor.http.Url
import kotlinx.datetime.LocalDate
import schemas.AdditionalMetadata
import schemas.manifest.InstallerManifest

object ManifestData {
    lateinit var installerUrl: Url
    lateinit var installerSha256: String
    lateinit var architecture: InstallerManifest.Installer.Architecture
    var installerType: InstallerManifest.InstallerType? = null
    var installerSwitches = InstallerManifest.InstallerSwitches()
    var installerLocale: String? = null
    var scope: InstallerManifest.Scope? = null
    var upgradeBehavior: InstallerManifest.UpgradeBehavior? = null
    var releaseDate: LocalDate? = null
    var installers = emptyList<InstallerManifest.Installer>()
    var fileExtensions: List<String>? = null
    var protocols: List<String>? = null
    var commands: List<String>? = null
    var installerSuccessCodes: List<Long>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null

    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
    var updateState: VersionUpdateState = VersionUpdateState.NewVersion
    var publisher: String? = null
    var packageName: String? = null
    var defaultLocale: String? = null
    var allVersions: List<String>? = null
    var msix: Msix? = null
    var msixBundle: MsixBundle? = null
    var msi: Msi? = null
    var zip: Zip? = null
    var gitHubDetection: GitHubDetection? = null
    var pageScraper: PageScraper? = null
    var additionalMetadata: AdditionalMetadata? = null

    var license: String? = null
    var shortDescription: String? = null
    var moniker: String? = null
    var publisherUrl: Url? = null
    var author: String? = null
    var packageUrl: Url? = null
    var licenseUrl: Url? = null
    var copyright: String? = null
    var copyrightUrl: Url? = null
    var tags: List<String>? = null
    var description: String? = null
    var releaseNotesUrl: Url? = null

    var skipAddInstaller = false
}
