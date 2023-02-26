package data

import io.ktor.http.URLBuilder
import io.ktor.http.Url
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.InstallerManifest

object InstallerManifestData : KoinComponent {
    private val schemas: Schemas by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val allManifestData: AllManifestData by inject()

    suspend fun addInstaller() = with(allManifestData) {
        val previousManifest = previousManifestData.remoteInstallerData.await()
        val previousInstaller = previousManifest?.installers?.getOrNull(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = msi?.productLanguage
                ?: allManifestData.installerLocale?.ifBlank { null }
                ?: previousInstaller?.installerLocale,
            platform = msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform
                ?: previousManifest?.platform?.map { it.toPerInstallerPlatform() },
            minimumOSVersion = msix?.minVersion,
            architecture = previousInstaller?.architecture ?: allManifestData.architecture,
            installerType = allManifestData.installerType ?: previousInstaller?.installerType,
            nestedInstallerType = zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousManifest?.nestedInstallerType?.toPerInstallerNestedInstallerType(),
            nestedInstallerFiles = (
                zip?.nestedInstallerFiles?.ifEmpty { null }
                    ?: previousInstaller?.nestedInstallerFiles
                    ?: previousManifest?.nestedInstallerFiles?.map { it.toPerInstallerNestedInstallerFiles() }
                )?.map { it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString()) },
            installerUrl = allManifestData.installerUrl,
            installerSha256 = (gitHubDetection?.sha256?.await() ?: installerSha256).uppercase(),
            signatureSha256 = msix?.signatureSha256
                ?: msixBundle?.signatureSha256,
            scope = allManifestData.scope
                ?: previousInstaller?.scope
                ?: previousManifest?.scope?.toPerScopeInstallerType(),
            packageFamilyName = msix?.packageFamilyName
                ?: msixBundle?.packageFamilyName
                ?: previousInstaller?.packageFamilyName
                ?: previousManifest?.packageFamilyName,
            installerSwitches = allManifestData.installerSwitches.takeUnless { it.areAllNullOrBlank() }
                ?: previousInstaller?.installerSwitches
                ?: previousManifest?.installerSwitches?.toPerInstallerSwitches(),
            upgradeBehavior = allManifestData.upgradeBehavior
                ?: previousInstaller?.upgradeBehavior
                ?: previousManifest?.upgradeBehavior?.toPerInstallerUpgradeBehaviour(),
            productCode = msi?.productCode
                ?: additionalMetadata?.productCode?.ifBlank { null }
                ?: previousManifest?.productCode
                ?: previousInstaller?.productCode,
            releaseDate = gitHubDetection?.releaseDate
                ?: additionalMetadata?.releaseDate
                ?: allManifestData.releaseDate,
            appsAndFeaturesEntries = additionalMetadata?.appsAndFeaturesEntries
                ?: previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.fillARPEntry()
                } ?: previousManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.toInstallerAppsAndFeaturesEntry().fillARPEntry()
            } ?: listOfNotNull(
                InstallerManifest.Installer.AppsAndFeaturesEntry().fillARPEntry().takeUnless { it.areAllNull() }
            ).ifEmpty { null },
        )
        when (msixBundle) {
            null -> installers += installer
            else -> {
                msixBundle?.packages?.forEach { individualPackage ->
                    individualPackage.processorArchitecture?.let { architecture ->
                        installers += installer.copy(
                            architecture = architecture,
                            platform = individualPackage.targetDeviceFamily?.map { it.toPerInstallerPlatform() },
                        )
                    }
                }
            }
        }
        resetValues()
    }

    private suspend fun InstallerManifest.Installer.AppsAndFeaturesEntry.fillARPEntry():
        InstallerManifest.Installer.AppsAndFeaturesEntry = with(allManifestData) {
        val remoteDefaultLocaleData = previousManifestData.remoteDefaultLocaleData.await()
        val arpDisplayName = msi?.productName ?: displayName
        val packageName = packageName ?: remoteDefaultLocaleData?.packageName
        val arpPublisher = msi?.manufacturer ?: publisher
        val publisher = publisher ?: remoteDefaultLocaleData?.publisher
        val displayVersion = msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != packageName) arpDisplayName?.updateVersionInString() else null,
            publisher = if (arpPublisher != publisher) arpPublisher else null,
            displayVersion = if (displayVersion != packageVersion) displayVersion else null,
            upgradeCode = msi?.upgradeCode ?: upgradeCode
        )
    }

    private fun String.updateVersionInString(): String = with(allManifestData) {
        return allVersions?.joinToString("|") { it }
            ?.let { replaceFirst(Regex(it), packageVersion) }
            ?: this@updateVersionInString
    }

    suspend fun createInstallerManifest(): String = with(allManifestData) {
        val remoteInstallerData = previousManifestData.remoteInstallerData.await()
        return getInstallerManifestBase().copy(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            installerLocale = installers.getDistinctOrNull { it.installerLocale }?.ifBlank { null }
                ?: remoteInstallerData?.installerLocale,
            platform = installers.getDistinctOrNull { it.platform }?.map { it.toManifestPlatform() }
                ?: remoteInstallerData?.platform,
            minimumOSVersion = installers.getDistinctOrNull { it.minimumOSVersion }?.ifBlank { null }
                ?: remoteInstallerData?.minimumOSVersion,
            installerType = installers.getDistinctOrNull { it.installerType }?.toManifestInstallerType()
                ?: remoteInstallerData?.installerType,
            nestedInstallerType = installers.getDistinctOrNull { it.nestedInstallerType }
                ?.toManifestNestedInstallerType() ?: remoteInstallerData?.nestedInstallerType,
            nestedInstallerFiles = installers.getDistinctOrNull { it.nestedInstallerFiles }
                ?.map { it.toManifestNestedInstallerFiles() } ?: remoteInstallerData?.nestedInstallerFiles,
            scope = installers.getDistinctOrNull { it.scope }?.toManifestScope() ?: remoteInstallerData?.scope,
            packageFamilyName = installers.getDistinctOrNull { it.packageFamilyName }
                ?: remoteInstallerData?.packageFamilyName,
            installModes = installModes?.ifEmpty { null } ?: remoteInstallerData?.installModes,
            installerSwitches = installers.getDistinctOrNull { it.installerSwitches }?.toManifestInstallerSwitches()
                ?: remoteInstallerData?.installerSwitches,
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: remoteInstallerData?.installerSuccessCodes,
            upgradeBehavior = installers.getDistinctOrNull { it.upgradeBehavior }?.toManifestUpgradeBehaviour()
                ?: remoteInstallerData?.upgradeBehavior,
            commands = commands?.ifEmpty { null } ?: remoteInstallerData?.commands,
            protocols = protocols?.ifEmpty { null } ?: remoteInstallerData?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null } ?: remoteInstallerData?.fileExtensions,
            releaseDate = installers.getDistinctOrNull { it.releaseDate },
            appsAndFeaturesEntries = when (installers.distinctBy { it.appsAndFeaturesEntries }.size) {
                0 -> remoteInstallerData?.appsAndFeaturesEntries
                1 -> installers.first().appsAndFeaturesEntries?.map { it.toManifestARPEntry() }
                else -> null
            },
            installers = installers.removeNonDistinctKeys()
                .sortedWith(compareBy({ it.installerLocale }, { it.architecture }, { it.installerType }, { it.scope })),
            manifestType = Schemas.installerManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private inline fun <T, R> List<T>.getDistinctOrNull(selector: (T) -> R?): R? {
        val distinctList = mapNotNull(selector).distinct()
        return when (distinctList.size) {
            1 -> distinctList.first()
            else -> null
        }
    }

    private suspend fun getInstallerManifestBase(): InstallerManifest = with(allManifestData) {
        return previousManifestData.remoteInstallerData.await() ?: InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            manifestType = Schemas.installerManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        )
    }

    private inline fun <T, R> Iterable<T>.takeIfNotDistinct(selector: (T) -> R): R? {
        val distinctValues = mapNotNull(selector).distinct()
        return if (distinctValues.size == 1) null else distinctValues.firstOrNull()
    }

    private fun List<InstallerManifest.Installer>.removeNonDistinctKeys():
        List<InstallerManifest.Installer> = with(allManifestData) {
        return map { installer ->
            installer.copy(
                installerLocale = installers.takeIfNotDistinct { it.installerLocale },
                platform = installers.takeIfNotDistinct { it.platform },
                minimumOSVersion = installers.takeIfNotDistinct { it.minimumOSVersion },
                installerType = installers.takeIfNotDistinct { it.installerType },
                nestedInstallerType = installers.takeIfNotDistinct { it.nestedInstallerType },
                nestedInstallerFiles = installers.takeIfNotDistinct { it.nestedInstallerFiles },
                scope = installers.takeIfNotDistinct { it.scope },
                packageFamilyName = installers.takeIfNotDistinct { it.packageFamilyName },
                releaseDate = installers.takeIfNotDistinct { it.releaseDate },
                upgradeBehavior = installers.takeIfNotDistinct { it.upgradeBehavior },
                installerSwitches = installers.takeIfNotDistinct { it.installerSwitches },
                appsAndFeaturesEntries = installers.takeIfNotDistinct { it.appsAndFeaturesEntries }
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

    private fun resetValues() = with(allManifestData) {
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
