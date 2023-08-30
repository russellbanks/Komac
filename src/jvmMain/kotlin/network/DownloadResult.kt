package network

import kotlinx.datetime.LocalDate
import schemas.manifest.InstallerManifest
import utils.Zip
import utils.msi.Msi
import utils.msix.Msix
import utils.msix.MsixBundle

data class DownloadResult(
        val releaseDate: LocalDate?,
        val scope: InstallerManifest.Scope?,
        val installerSha256: String,
        val installerType: InstallerManifest.InstallerType?,
        val upgradeBehaviour: InstallerManifest.UpgradeBehavior?,
        val architecture: InstallerManifest.Installer.Architecture,
        val productCode: String?,
        val publisherDisplayName: String?,
        val msix: Msix?,
        val msixBundle: MsixBundle?,
        val msi: Msi?,
        val zip: Zip?
)
