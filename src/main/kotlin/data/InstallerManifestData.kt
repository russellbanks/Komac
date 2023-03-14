package data

import extensions.IterableExtensions.getDistinctOrNull
import extensions.IterableExtensions.takeIfNotDistinct
import io.ktor.http.URLBuilder
import io.ktor.http.Url
import schemas.Schemas
import schemas.manifest.InstallerManifest
import utils.ManifestUtils.updateVersionInString

object InstallerManifestData {
    suspend fun addInstaller(
        allManifestData: AllManifestData,
        previousManifestData: PreviousManifestData?
    ) = with(allManifestData) {
        val previousManifest = previousManifestData?.remoteInstallerData?.await()
        val previousInstaller = previousManifest?.installers?.getOrNull(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = msi?.productLanguage
                ?: installerLocale?.ifBlank { null }
                ?: previousInstaller?.installerLocale,
            platform = msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform
                ?: previousManifest?.platform?.map { it.toPerInstallerPlatform() },
            minimumOSVersion = msix?.minVersion,
            architecture = previousInstaller?.architecture ?: architecture,
            installerType = installerType ?: previousInstaller?.installerType,
            nestedInstallerType = zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousManifest?.nestedInstallerType?.toPerInstallerNestedInstallerType(),
            nestedInstallerFiles = (
                zip?.nestedInstallerFiles?.ifEmpty { null }
                    ?: previousInstaller?.nestedInstallerFiles
                    ?: previousManifest?.nestedInstallerFiles?.map { it.toPerInstallerNestedInstallerFiles() }
                )?.map {
                it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
            },
            installerUrl = installerUrl,
            installerSha256 = (gitHubDetection?.sha256?.await() ?: installerSha256).uppercase(),
            signatureSha256 = msix?.signatureSha256 ?: msixBundle?.signatureSha256,
            scope = scope ?: previousInstaller?.scope ?: previousManifest?.scope?.toPerScopeInstallerType(),
            packageFamilyName = msix?.packageFamilyName
                ?: msixBundle?.packageFamilyName
                ?: previousInstaller?.packageFamilyName
                ?: previousManifest?.packageFamilyName,
            installerSwitches = installerSwitches.takeUnless { it.areAllNullOrBlank() }
                ?: previousInstaller?.installerSwitches
                ?: previousManifest?.installerSwitches?.toPerInstallerSwitches(),
            upgradeBehavior = upgradeBehavior
                ?: previousInstaller?.upgradeBehavior
                ?: previousManifest?.upgradeBehavior?.toPerInstallerUpgradeBehaviour(),
            productCode = msi?.productCode
                ?: additionalMetadata?.productCode?.ifBlank { null }
                ?: previousManifest?.productCode
                ?: previousInstaller?.productCode,
            releaseDate = gitHubDetection?.releaseDate ?: additionalMetadata?.releaseDate ?: releaseDate,
            appsAndFeaturesEntries = additionalMetadata?.appsAndFeaturesEntries
                ?: previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.fillARPEntry(allManifestData, previousManifestData)
                } ?: previousManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.toInstallerAppsAndFeaturesEntry().fillARPEntry(
                    allManifestData = allManifestData,
                    previousManifestData = previousManifestData
                )
            } ?: listOfNotNull(
                InstallerManifest.Installer.AppsAndFeaturesEntry()
                    .fillARPEntry(allManifestData, previousManifestData)
                    .takeUnless { it.areAllNull() }
            ).ifEmpty { null },
        )
        if (msixBundle == null) {
            installers += installer
        } else {
            msixBundle?.packages?.forEach { individualPackage ->
                individualPackage.processorArchitecture?.let { architecture ->
                    installers += installer.copy(
                        architecture = architecture,
                        platform = individualPackage.targetDeviceFamily?.map { it.toPerInstallerPlatform() },
                    )
                }
            }
        }
        resetValues(allManifestData)
    }

    private suspend fun InstallerManifest.Installer.AppsAndFeaturesEntry.fillARPEntry(
        allManifestData: AllManifestData,
        previousManifestData: PreviousManifestData?
    ): InstallerManifest.Installer.AppsAndFeaturesEntry = with(allManifestData) {
        val remoteDefaultLocaleData = previousManifestData?.remoteDefaultLocaleData?.await()
        val arpDisplayName = msi?.productName ?: displayName
        val packageName = packageName ?: remoteDefaultLocaleData?.packageName
        val arpPublisher = msi?.manufacturer ?: publisher
        val publisher = publisher ?: remoteDefaultLocaleData?.publisher
        val displayVersion = msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != packageName) {
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
        allManifestData: AllManifestData,
        previousInstallerManifest: InstallerManifest?,
        manifestOverride: String? = null
    ): String = with(allManifestData) {
        return getInstallerManifestBase(allManifestData, previousInstallerManifest).copy(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            installerLocale = installers.getDistinctOrNull { it.installerLocale }?.ifBlank { null }
                ?: previousInstallerManifest?.installerLocale,
            platform = installers.getDistinctOrNull { it.platform }?.map { it.toManifestPlatform() }
                ?: previousInstallerManifest?.platform,
            minimumOSVersion = installers.getDistinctOrNull { it.minimumOSVersion }?.ifBlank { null }
                ?: previousInstallerManifest?.minimumOSVersion,
            installerType = installers.getDistinctOrNull { it.installerType }?.toManifestInstallerType()
                ?: previousInstallerManifest?.installerType,
            nestedInstallerType = installers.getDistinctOrNull { it.nestedInstallerType }
                ?.toManifestNestedInstallerType()
                ?: previousInstallerManifest?.nestedInstallerType,
            nestedInstallerFiles = (
                installers.getDistinctOrNull { it.nestedInstallerFiles }
                    ?.map { it.toManifestNestedInstallerFiles() }
                    ?: previousInstallerManifest?.nestedInstallerFiles
                )?.map {
                it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
            },
            scope = installers.getDistinctOrNull { it.scope }?.toManifestScope() ?: previousInstallerManifest?.scope,
            packageFamilyName = installers.getDistinctOrNull { it.packageFamilyName }
                ?: previousInstallerManifest?.packageFamilyName,
            installModes = installModes?.ifEmpty { null } ?: previousInstallerManifest?.installModes,
            installerSwitches = installers.getDistinctOrNull { it.installerSwitches }?.toManifestInstallerSwitches()
                ?: previousInstallerManifest?.installerSwitches,
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: previousInstallerManifest?.installerSuccessCodes,
            upgradeBehavior = installers.getDistinctOrNull { it.upgradeBehavior }?.toManifestUpgradeBehaviour()
                ?: previousInstallerManifest?.upgradeBehavior,
            commands = commands?.ifEmpty { null } ?: previousInstallerManifest?.commands,
            protocols = protocols?.ifEmpty { null } ?: previousInstallerManifest?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null } ?: previousInstallerManifest?.fileExtensions,
            releaseDate = installers.getDistinctOrNull { it.releaseDate },
            appsAndFeaturesEntries = when (installers.distinctBy { it.appsAndFeaturesEntries }.size) {
                0 -> previousInstallerManifest?.appsAndFeaturesEntries
                1 -> installers.first().appsAndFeaturesEntries?.map { it.toManifestARPEntry() }
                else -> null
            },
            installers = installers.removeNonDistinctKeys(installers)
                .sortedWith(
                    compareBy(
                        InstallerManifest.Installer::installerLocale,
                        InstallerManifest.Installer::architecture,
                        InstallerManifest.Installer::installerType,
                        InstallerManifest.Installer::scope
                    )
                ),
            manifestType = Schemas.installerManifestType,
            manifestVersion = manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private fun getInstallerManifestBase(
        allManifestData: AllManifestData,
        previousManifestData: InstallerManifest?
    ): InstallerManifest = with(allManifestData) {
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
            installerUrl = Url(URLBuilder())
        )
    }

    private fun resetValues(allManifestData: AllManifestData) = with(allManifestData) {
        installerLocale = null
        scope = null
        installerSwitches = InstallerManifest.Installer.InstallerSwitches()
        upgradeBehavior = null
        releaseDate = null
        msi?.resetExceptShared()
        msix?.resetExceptShared()
        msixBundle?.resetExceptShared()
        zip = null
        gitHubDetection?.releaseDate = null
    }
}
