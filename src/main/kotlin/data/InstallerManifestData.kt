package data

import io.ktor.http.Url
import schemas.Schemas
import schemas.installerSorter
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest
import utils.ManifestUtils.updateVersionInString
import utils.getDistinctOrNull
import utils.msi.Msi
import utils.takeIfNotDistinct

object InstallerManifestData {
    fun addInstaller() = with(ManifestData) {
        val previousInstallerManifest = PreviousManifestData.installerManifest
        val previousDefaultLocaleManifest = PreviousManifestData.defaultLocaleManifest
        val previousInstaller = previousInstallerManifest?.installers?.getOrNull(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
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
            productCode = msi?.productCode
                ?: additionalMetadata?.productCode?.ifBlank { null }
                ?: (previousInstallerManifest?.productCode ?: previousInstaller?.productCode)
                    ?.updateVersionInString(allVersions, packageVersion),
            releaseDate = gitHubDetection?.releaseDate ?: additionalMetadata?.releaseDate ?: releaseDate,
            appsAndFeaturesEntries = additionalMetadata?.appsAndFeaturesEntries
                ?: previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.fillARPEntry(
                        packageName, packageVersion, allVersions, msi, previousDefaultLocaleManifest
                    )
                } ?: previousInstallerManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.fillARPEntry(
                    packageName, packageVersion, allVersions, msi, previousDefaultLocaleManifest
                )
            } ?: listOfNotNull(
                InstallerManifest.AppsAndFeaturesEntry()
                    .fillARPEntry(packageName, packageVersion, allVersions, msi, previousDefaultLocaleManifest)
                    .takeUnless(InstallerManifest.AppsAndFeaturesEntry::areAllNull)
            ).ifEmpty { null },
        )
        if (msixBundle == null) {
            installers += installer
        } else {
            msixBundle?.packages?.forEach { individualPackage ->
                individualPackage.processorArchitecture?.let { architecture ->
                    installers += installer.copy(
                        architecture = architecture,
                        platform = individualPackage.targetDeviceFamily,
                    )
                }
            }
        }
        resetValues(ManifestData)
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

    fun createInstallerManifest(manifestOverride: String? = null): InstallerManifest = with(ManifestData) {
        val previousInstallerManifest = PreviousManifestData.installerManifest
        return getInstallerManifestBase(previousInstallerManifest).copy(
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
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        )
    }

    private fun getInstallerManifestBase(
        previousManifestData: InstallerManifest?
    ): InstallerManifest = with(ManifestData) {
        return previousManifestData ?: InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            manifestType = Schemas.installerManifestType,
            manifestVersion = Schemas.manifestVersion
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
                releaseDate = installers.takeIfNotDistinct(installer.releaseDate) { it.releaseDate },
                upgradeBehavior = installers.takeIfNotDistinct(installer.upgradeBehavior) { it.upgradeBehavior },
                installerSwitches = installers.takeIfNotDistinct(installer.installerSwitches) { it.installerSwitches },
                appsAndFeaturesEntries = installers
                    .takeIfNotDistinct(installer.appsAndFeaturesEntries) { it.appsAndFeaturesEntries }
            )
        }
    }

    private fun getInstallerBase(previousInstaller: InstallerManifest.Installer?): InstallerManifest.Installer {
        return previousInstaller ?: InstallerManifest.Installer(
            architecture = InstallerManifest.Installer.Architecture.NEUTRAL,
            installerSha256 = "",
            installerUrl = Url("")
        )
    }

    private fun resetValues(allManifestData: ManifestData) = with(allManifestData) {
        installerLocale = null
        scope = null
        installerSwitches = InstallerManifest.InstallerSwitches()
        upgradeBehavior = null
        releaseDate = null
        msi?.resetExceptShared()
        msix = null
        zip = null
        gitHubDetection?.releaseDate = null
    }
}
