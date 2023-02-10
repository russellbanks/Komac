package data

import io.ktor.http.URLBuilder
import io.ktor.http.Url
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.manifest.InstallerManifest
import java.time.LocalDate

@Single
class InstallerManifestData : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    lateinit var installerUrl: Url
    lateinit var installerSha256: String
    lateinit var architecture: InstallerManifest.Installer.Architecture
    var installerType: InstallerManifest.Installer.InstallerType? = null
    var installerSwitches = InstallerManifest.Installer.InstallerSwitches()
    var installerLocale: String? = null
    var productCode: String? = null
    var scope: InstallerManifest.Installer.Scope? = null
    var upgradeBehavior: InstallerManifest.Installer.UpgradeBehavior? = null
    var releaseDate: LocalDate? = null
    var installers = listOf<InstallerManifest.Installer>()
    var fileExtensions: List<String>? = null
    var protocols: List<String>? = null
    var commands: List<String>? = null
    var installerSuccessCodes: List<Long>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null
    private val schemas: Schemas by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun addInstaller() {
        val previousManifest = previousManifestData.remoteInstallerData.await()
        val previousInstaller = previousManifest?.installers?.getOrNull(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = sharedManifestData.msi?.productLanguage
                ?: installerLocale?.ifBlank { null }
                ?: previousInstaller?.installerLocale,
            platform = sharedManifestData.msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform
                ?: previousManifest?.platform?.map { it.toPerInstallerPlatform() },
            minimumOSVersion = sharedManifestData.msix?.minVersion,
            architecture = previousInstaller?.architecture ?: architecture,
            installerType = if (installerType != null) installerType else previousInstaller?.installerType,
            nestedInstallerType = sharedManifestData.zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousManifest?.nestedInstallerType?.toPerInstallerNestedInstallerType(),
            nestedInstallerFiles = (sharedManifestData.zip?.nestedInstallerFiles?.ifEmpty { null }
                ?: previousInstaller?.nestedInstallerFiles
                ?: previousManifest?.nestedInstallerFiles?.map { it.toPerInstallerNestedInstallerFiles() })
                ?.map { it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString()) },
            installerUrl = installerUrl,
            installerSha256 = (sharedManifestData.gitHubDetection?.sha256?.await() ?: installerSha256).uppercase(),
            signatureSha256 = sharedManifestData.msix?.signatureSha256
                ?: sharedManifestData.msixBundle?.signatureSha256,
            scope = scope ?: previousInstaller?.scope ?: previousManifest?.scope?.toPerScopeInstallerType(),
            packageFamilyName = sharedManifestData.msix?.packageFamilyName
                ?: sharedManifestData.msixBundle?.packageFamilyName
                ?: previousInstaller?.packageFamilyName
                ?: previousManifest?.packageFamilyName,
            installerSwitches = installerSwitches.takeUnless { it.areAllNullOrBlank() }
                ?: previousInstaller?.installerSwitches
                ?: previousManifest?.installerSwitches?.toPerInstallerSwitches(),
            upgradeBehavior = upgradeBehavior
                ?: previousInstaller?.upgradeBehavior
                ?: previousManifest?.upgradeBehavior?.toPerInstallerUpgradeBehaviour(),
            productCode = sharedManifestData.msi?.productCode
                ?: sharedManifestData.additionalMetadata?.productCode?.ifBlank { null }
                ?: productCode?.ifBlank { null }
                ?: previousManifest?.productCode
                ?: previousInstaller?.productCode,
            releaseDate = sharedManifestData.gitHubDetection?.releaseDate?.await()
                ?: sharedManifestData.additionalMetadata?.releaseDate
                ?: releaseDate,
            appsAndFeaturesEntries = sharedManifestData.additionalMetadata?.appsAndFeaturesEntries
                ?: previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.fillARPEntry()
                } ?: previousManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                    appsAndFeaturesEntry.toInstallerAppsAndFeaturesEntry().fillARPEntry()
                } ?: listOfNotNull(
                    InstallerManifest.Installer.AppsAndFeaturesEntry().fillARPEntry().takeUnless { it.areAllNull() }
                ).ifEmpty { null },
        )
        when (sharedManifestData.msixBundle) {
            null -> installers += installer
            else -> {
                sharedManifestData.msixBundle?.packages?.forEach { individualPackage ->
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
        InstallerManifest.Installer.AppsAndFeaturesEntry {
        val remoteDefaultLocaleData = previousManifestData.remoteDefaultLocaleData.await()
        val arpDisplayName = sharedManifestData.msi?.productName ?: displayName
        val packageName = sharedManifestData.packageName ?: remoteDefaultLocaleData?.packageName
        val arpPublisher = sharedManifestData.msi?.manufacturer ?: publisher
        val publisher = sharedManifestData.publisher ?: remoteDefaultLocaleData?.publisher
        val displayVersion = sharedManifestData.msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != packageName) arpDisplayName?.updateVersionInString() else null,
            publisher = if (arpPublisher != publisher) arpPublisher else null,
            displayVersion = if (displayVersion != sharedManifestData.packageVersion) displayVersion else null,
            upgradeCode = sharedManifestData.msi?.upgradeCode ?: upgradeCode
        )
    }

    private fun String.updateVersionInString(): String {
        return sharedManifestData.allVersions?.joinToString("|") { it }
            ?.let { replaceFirst(Regex(it), sharedManifestData.packageVersion) }
            ?: this
    }

    suspend fun createInstallerManifest(): String {
        val remoteInstallerData = previousManifestData.remoteInstallerData.await()
        return getInstallerManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            installerLocale = when (installers.mapNotNull { it.installerLocale }.distinct().size) {
                1 -> installers.firstNotNullOf { it.installerLocale }.ifBlank { null }
                else -> remoteInstallerData?.installerLocale
            },
            platform = when (installers.mapNotNull { it.platform }.distinct().size) {
                1 -> installers.firstNotNullOf { it.platform }.map { it.toManifestPlatform() }
                else -> remoteInstallerData?.platform
            },
            minimumOSVersion = when (installers.mapNotNull { it.minimumOSVersion }.distinct().size) {
                1 -> installers.firstNotNullOf { it.minimumOSVersion }
                else -> remoteInstallerData?.minimumOSVersion
            },
            installerType = when (installers.mapNotNull { it.installerType }.distinct().size) {
                1 -> installers.firstNotNullOf { it.installerType }.toManifestInstallerType()
                else -> remoteInstallerData?.installerType
            },
            scope = when (installers.mapNotNull { it.scope }.distinct().size) {
                1 -> installers.firstNotNullOf { it.scope }.toManifestScope()
                else -> remoteInstallerData?.scope
            },
            packageFamilyName = when (installers.mapNotNull { it.packageFamilyName }.distinct().size) {
                1 -> installers.firstNotNullOf { it.packageFamilyName }
                else -> remoteInstallerData?.packageFamilyName
            },
            installModes = installModes?.ifEmpty { null } ?: remoteInstallerData?.installModes,
            installerSwitches = when (installers.mapNotNull { it.installerSwitches }.distinct().size) {
                1 -> installers.firstNotNullOf { it.installerSwitches }.toManifestInstallerSwitches()
                else -> remoteInstallerData?.installerSwitches
            },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: remoteInstallerData?.installerSuccessCodes,
            upgradeBehavior = when (installers.mapNotNull { it.upgradeBehavior }.distinct().size) {
                1 -> installers.firstNotNullOf { it.upgradeBehavior }.toManifestUpgradeBehaviour()
                else -> remoteInstallerData?.upgradeBehavior
            },
            commands = commands?.ifEmpty { null } ?: remoteInstallerData?.commands,
            protocols = protocols?.ifEmpty { null } ?: remoteInstallerData?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null }
                ?: remoteInstallerData?.fileExtensions,
            releaseDate = when (installers.mapNotNull { it.releaseDate }.distinct().size) {
                1 -> installers.map { it.releaseDate }.first()
                else -> null
            },
            appsAndFeaturesEntries = when (installers.distinctBy { it.appsAndFeaturesEntries }.size) {
                0 -> remoteInstallerData?.appsAndFeaturesEntries
                1 -> {
                    installers
                        .first()
                        .appsAndFeaturesEntries
                        ?.map { it.toManifestARPEntry() }
                }
                else -> null
            },
            installers = installers.removeNonDistinctKeys()
                .sortedWith(compareBy({ it.installerLocale }, { it.architecture }, { it.installerType }, { it.scope })),
            manifestType = Schemas.installerManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private suspend fun getInstallerManifestBase(): InstallerManifest {
        return previousManifestData.remoteInstallerData.await() ?: InstallerManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            manifestType = Schemas.installerManifestType,
            manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion
        )
    }

    private fun List<InstallerManifest.Installer>.removeNonDistinctKeys(): List<InstallerManifest.Installer> {
        return map { installer ->
            installer.copy(
                installerLocale = if (installers.mapNotNull { it.installerLocale }.distinct().size == 1) {
                    null
                } else {
                    installer.installerLocale
                },
                platform = if (installers.mapNotNull { it.platform }.distinct().size == 1) null else installer.platform,
                minimumOSVersion = if (installers.mapNotNull { it.minimumOSVersion }.distinct().size == 1) {
                    null
                } else {
                    installer.minimumOSVersion
                },
                installerType = if (installers.mapNotNull { it.installerType }.distinct().size == 1) {
                    null
                } else {
                    installer.installerType
                },
                scope = if (installers.mapNotNull { it.scope }.distinct().size == 1) null else installer.scope,
                packageFamilyName = if (installers.mapNotNull { it.packageFamilyName }.distinct().size == 1) {
                    null
                } else {
                    installer.packageFamilyName
                },
                releaseDate = if (installers.mapNotNull { it.releaseDate }.distinct().size == 1) {
                    null
                } else {
                    installer.releaseDate
                },
                upgradeBehavior = if (installers.mapNotNull { it.upgradeBehavior }.distinct().size == 1) {
                    null
                } else {
                    installer.upgradeBehavior
                },
                installerSwitches = if (installers.mapNotNull { it.installerSwitches }.distinct().size == 1) {
                    null
                } else {
                    installer.installerSwitches
                },
                appsAndFeaturesEntries = if (installers.distinctBy { it.appsAndFeaturesEntries }.size == 1) {
                    null
                } else {
                    installer.appsAndFeaturesEntries
                }
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

    private suspend fun resetValues() = coroutineScope {
        installerLocale = null
        scope = null
        installerSwitches = InstallerManifest.Installer.InstallerSwitches()
        upgradeBehavior = null
        productCode = null
        releaseDate = null
        sharedManifestData.msi?.resetExceptShared()
        sharedManifestData.msix?.resetExceptShared()
        sharedManifestData.msixBundle?.resetExceptShared()
        sharedManifestData.zip = null
        sharedManifestData.gitHubDetection?.releaseDate = async { null }
    }
}
