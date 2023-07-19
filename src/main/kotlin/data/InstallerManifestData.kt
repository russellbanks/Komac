package data

import github.GitHubDetection
import io.ktor.http.Url
import kotlinx.datetime.LocalDate
import schemas.AdditionalMetadata
import schemas.Schemas
import schemas.installerSorter
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest
import utils.ManifestUtils.updateVersionInString
import utils.Zip
import utils.getDistinctOrNull
import utils.msi.Msi
import utils.msix.Msix
import utils.msix.MsixBundle
import utils.takeIfNotDistinct

object InstallerManifestData {
    suspend fun addInstaller(
        packageVersion: String,
        installerUrl: Url,
        installerSha256: String,
        installerType: InstallerManifest.InstallerType?,
        installerLocale: String? = null,
        scope: InstallerManifest.Scope? = null,
        releaseDate: LocalDate? = null,
        packageName: String? = null,
        installerSwitches: InstallerManifest.InstallerSwitches = InstallerManifest.InstallerSwitches(),
        allVersions: List<String>? = null,
        upgradeBehavior: InstallerManifest.UpgradeBehavior? = null,
        installers: List<InstallerManifest.Installer>,
        architecture: InstallerManifest.Installer.Architecture,
        additionalMetadata: AdditionalMetadata? = null,
        productCode: String? = null,
        msix: Msix?,
        msixBundle: MsixBundle?,
        msi: Msi?,
        zip: Zip?,
        gitHubDetection: GitHubDetection?,
        previousManifestData: PreviousManifestData?,
        onAddInstaller: (InstallerManifest.Installer) -> Unit
    ) {
        val previousInstallerManifest = previousManifestData?.installerManifest?.await()
        val previousInstaller = previousInstallerManifest?.installers?.getOrNull(installers.size)
        val installer = InstallerManifest.getInstallerBase(previousInstaller).copy(
            installerLocale = msi?.productLanguage
                ?: installerLocale?.ifBlank { null }
                ?: previousInstaller?.installerLocale,
            platform = msix?.targetDeviceFamily?.let(::listOf)
                ?: previousInstaller?.platform
                ?: previousInstallerManifest?.platform,
            minimumOSVersion = msix?.minVersion,
            architecture = previousInstaller?.architecture ?: architecture,
            installerType = installerType ?: previousInstaller?.installerType,
            nestedInstallerType = zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousInstallerManifest?.nestedInstallerType,
            nestedInstallerFiles = (
                zip?.nestedInstallerFiles?.ifEmpty { null }
                    ?: previousInstaller?.nestedInstallerFiles
                    ?: previousInstallerManifest?.nestedInstallerFiles
                )?.map {
                    it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
                },
            installerUrl = installerUrl,
            installerSha256 = (gitHubDetection?.sha256 ?: installerSha256).uppercase(),
            signatureSha256 = (msix?.signatureSha256 ?: msixBundle?.signatureSha256)?.uppercase(),
            scope = scope ?: previousInstaller?.scope ?: previousInstallerManifest?.scope,
            packageFamilyName = msix?.packageFamilyName
                ?: msixBundle?.packageFamilyName
                ?: previousInstaller?.packageFamilyName
                ?: previousInstallerManifest?.packageFamilyName,
            installerSwitches = installerSwitches.takeUnless(InstallerManifest.InstallerSwitches::areAllNullOrBlank)
                ?: previousInstaller?.installerSwitches
                ?: previousInstallerManifest?.installerSwitches,
            upgradeBehavior = upgradeBehavior
                ?: previousInstaller?.upgradeBehavior
                ?: previousInstallerManifest?.upgradeBehavior,
            productCode = productCode
                ?: additionalMetadata?.productCode?.ifBlank { null }
                ?: (previousInstallerManifest?.productCode ?: previousInstaller?.productCode)
                    ?.updateVersionInString(allVersions, packageVersion),
            releaseDate = additionalMetadata?.releaseDate ?: gitHubDetection?.releaseDate ?: releaseDate,
            appsAndFeaturesEntries = additionalMetadata?.appsAndFeaturesEntries
                ?: previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.fillARPEntry(
                        packageName, packageVersion, allVersions, msi, previousManifestData?.defaultLocaleManifest
                    )
                } ?: previousInstallerManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.fillARPEntry(
                    packageName, packageVersion, allVersions, msi, previousManifestData?.defaultLocaleManifest
                )
            } ?: listOfNotNull(
                InstallerManifest.AppsAndFeaturesEntry()
                    .fillARPEntry(packageName, packageVersion, allVersions, msi, previousManifestData?.defaultLocaleManifest)
                    .takeUnless(InstallerManifest.AppsAndFeaturesEntry::areAllNull)
            ).ifEmpty { null },
        )
        if (msixBundle == null) {
            onAddInstaller(installer)
        } else {
            msixBundle.packages?.forEach { individualPackage ->
                individualPackage.processorArchitecture?.let { architecture ->
                    onAddInstaller(
                        installer.copy(
                            architecture = architecture,
                            platform = individualPackage.targetDeviceFamily,
                        )
                    )
                }
            }
        }
    }

    private fun InstallerManifest.AppsAndFeaturesEntry.fillARPEntry(
        packageName: String?,
        packageVersion: String,
        allVersions: List<String>?,
        msi: Msi?,
        previousDefaultLocaleData: DefaultLocaleManifest?
    ): InstallerManifest.AppsAndFeaturesEntry {
        val arpDisplayName = msi?.productName ?: displayName
        val name = packageName ?: previousDefaultLocaleData?.packageName
        val arpPublisher = msi?.manufacturer ?: publisher
        val publisher = publisher ?: previousDefaultLocaleData?.publisher
        val displayVersion = msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != name) {
                arpDisplayName?.updateVersionInString(allVersions, packageVersion)
            } else {
                null
            },
            publisher = if (arpPublisher != publisher) arpPublisher else null,
            displayVersion = if (displayVersion != packageVersion) displayVersion else null,
            upgradeCode = msi?.upgradeCode ?: upgradeCode
        )
    }

    fun createInstallerManifest(
        packageIdentifier: String,
        packageVersion: String,
        commands: List<String>? = null,
        fileExtensions: List<String>? = null,
        protocols: List<String>? = null,
        installerSuccessCodes: List<Long>? = null,
        installModes: List<InstallerManifest.InstallModes>? = null,
        allVersions: List<String>?,
        installers: List<InstallerManifest.Installer>,
        previousInstallerManifest: InstallerManifest?,
        manifestOverride: String
    ): InstallerManifest {
        return InstallerManifest.getBase(previousInstallerManifest, packageIdentifier, packageVersion).copy(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            installerLocale = installers.getDistinctOrNull(InstallerManifest.Installer::installerLocale)
                ?.ifBlank { null }
                ?: previousInstallerManifest?.installerLocale,
            platform = installers.getDistinctOrNull(InstallerManifest.Installer::platform)
                ?: previousInstallerManifest?.platform,
            minimumOSVersion = installers.getDistinctOrNull(InstallerManifest.Installer::minimumOSVersion)
                ?.ifBlank { null },
            installerType = installers.getDistinctOrNull(InstallerManifest.Installer::installerType)
                ?: previousInstallerManifest?.installerType,
            nestedInstallerType = installers.getDistinctOrNull(InstallerManifest.Installer::nestedInstallerType)
                ?: previousInstallerManifest?.nestedInstallerType,
            nestedInstallerFiles = (
                installers.getDistinctOrNull(InstallerManifest.Installer::nestedInstallerFiles)
                    ?: previousInstallerManifest?.nestedInstallerFiles
            )?.map {
                it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
            },
            scope = installers.getDistinctOrNull(InstallerManifest.Installer::scope)
                ?: previousInstallerManifest?.scope,
            packageFamilyName = installers.getDistinctOrNull(InstallerManifest.Installer::packageFamilyName)
                ?: previousInstallerManifest?.packageFamilyName,
            productCode = installers.getDistinctOrNull(InstallerManifest.Installer::productCode),
            installModes = installModes?.ifEmpty { null }
                ?: previousInstallerManifest?.installModes,
            installerSwitches = installers.getDistinctOrNull(InstallerManifest.Installer::installerSwitches)
                ?: previousInstallerManifest?.installerSwitches,
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: previousInstallerManifest?.installerSuccessCodes,
            upgradeBehavior = installers.getDistinctOrNull(InstallerManifest.Installer::upgradeBehavior)
                ?: previousInstallerManifest?.upgradeBehavior,
            commands = commands?.ifEmpty { null } ?: previousInstallerManifest?.commands,
            protocols = protocols?.ifEmpty { null } ?: previousInstallerManifest?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null } ?: previousInstallerManifest?.fileExtensions,
            releaseDate = installers.getDistinctOrNull { it.releaseDate },
            appsAndFeaturesEntries = when (installers.distinctBy { it.appsAndFeaturesEntries }.size) {
                0 -> previousInstallerManifest?.appsAndFeaturesEntries
                1 -> installers.first().appsAndFeaturesEntries
                else -> null
            },
            installers = installers.removeNonDistinctKeys(installers).sortedWith(installerSorter),
            manifestType = Schemas.installerManifestType,
            manifestVersion = manifestOverride
        )
    }

    private fun List<InstallerManifest.Installer>.removeNonDistinctKeys(installers: List<InstallerManifest.Installer>):
        List<InstallerManifest.Installer> {
        return map { installer ->
            installer.copy(
                installerLocale = installers.takeIfNotDistinct(installer.installerLocale) { it.installerLocale },
                platform = installers.takeIfNotDistinct(installer.platform) { it.platform },
                minimumOSVersion = installers.takeIfNotDistinct(installer.minimumOSVersion) { it.minimumOSVersion },
                installerType = installers.takeIfNotDistinct(installer.installerType) { it.installerType },
                nestedInstallerType = installers
                    .takeIfNotDistinct(installer.nestedInstallerType) { it.nestedInstallerType },
                nestedInstallerFiles = installers
                    .takeIfNotDistinct(installer.nestedInstallerFiles) { it.nestedInstallerFiles },
                scope = installers.takeIfNotDistinct(installer.scope) { it.scope },
                packageFamilyName = installers.takeIfNotDistinct(installer.packageFamilyName) { it.packageFamilyName },
                productCode = installers.takeIfNotDistinct(installer.productCode) { it.productCode },
                releaseDate = installers.takeIfNotDistinct(installer.releaseDate) { it.releaseDate },
                upgradeBehavior = installers.takeIfNotDistinct(installer.upgradeBehavior) { it.upgradeBehavior },
                installerSwitches = installers.takeIfNotDistinct(installer.installerSwitches) { it.installerSwitches },
                appsAndFeaturesEntries = installers
                    .takeIfNotDistinct(installer.appsAndFeaturesEntries) { it.appsAndFeaturesEntries }
            )
        }
    }
}
