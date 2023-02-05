package data

import io.ktor.http.URLBuilder
import io.ktor.http.Url
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schemas
import schemas.SchemasImpl
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
    var upgradeBehavior: InstallerManifest.UpgradeBehavior? = null
    var releaseDate: LocalDate? = null
    var installers = listOf<InstallerManifest.Installer>()
    var fileExtensions: List<String>? = null
    var protocols: List<String>? = null
    var commands: List<String>? = null
    var installerSuccessCodes: List<Long>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null
    private val schemasImpl: SchemasImpl by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun addInstaller() {
        previousManifestData.remoteInstallerDataJob.join()
        val previousManifest = previousManifestData.remoteInstallerData
        val previousInstaller = previousManifest?.installers?.getOrNull(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = sharedManifestData.msi?.productLanguage
                ?: installerLocale?.ifBlank { null }
                ?: previousInstaller?.installerLocale,
            platform = sharedManifestData.msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform
                ?: previousManifest?.platform?.map { it.toPerInstallerPlatform() },
            minimumOSVersion = sharedManifestData.msix?.minVersion,
            architecture = if (::architecture.isInitialized) architecture else previousInstaller?.architecture!!,
            installerType = if (installerType != null) installerType else previousInstaller?.installerType,
            nestedInstallerType = sharedManifestData.zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousManifest?.nestedInstallerType?.toPerInstallerNestedInstallerType(),
            nestedInstallerFiles = sharedManifestData.zip?.nestedInstallerFiles
                ?.ifEmpty { null }
                ?: previousInstaller?.nestedInstallerFiles
                ?: previousManifest?.nestedInstallerFiles?.map { it.toPerInstallerNestedInstallerFiles() },
            installerUrl = installerUrl,
            installerSha256 = (sharedManifestData.gitHubDetection?.sha256?.await() ?: installerSha256).uppercase(),
            signatureSha256 = sharedManifestData.msix?.signatureSha256
                ?: sharedManifestData.msixBundle?.signatureSha256,
            scope = if (sharedManifestData.msi?.allUsers != null) {
                sharedManifestData.msi?.allUsers?.toInstallerScope()
            } else {
                scope ?: previousInstaller?.scope ?: previousManifest?.scope?.toPerScopeInstallerType()
            },
            installerSwitches = installerSwitches
                .takeUnless { it.areAllNullOrBlank() }
                ?: previousInstaller?.installerSwitches
                ?: previousManifest?.installerSwitches?.toPerInstallerSwitches(),
            upgradeBehavior = upgradeBehavior?.toPerInstallerUpgradeBehaviour()
                ?: previousInstaller?.upgradeBehavior
                ?: previousManifest?.upgradeBehavior?.toPerInstallerUpgradeBehaviour(),
            productCode = sharedManifestData.msi?.productCode
                ?: sharedManifestData.additionalMetadata?.productCode?.ifBlank { null }
                ?: productCode?.ifBlank { null }
                ?: previousManifest?.productCode,
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

    private fun InstallerManifest.Installer.AppsAndFeaturesEntry.fillARPEntry():
        InstallerManifest.Installer.AppsAndFeaturesEntry {
        val arpDisplayName = sharedManifestData.msi?.productName ?: displayName
        val packageName = sharedManifestData.packageName ?: previousManifestData.remoteDefaultLocaleData?.packageName
        val arpPublisher = sharedManifestData.msi?.manufacturer ?: publisher
        val publisher = sharedManifestData.publisher ?: previousManifestData.remoteDefaultLocaleData?.publisher
        val displayVersion = sharedManifestData.msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != packageName) arpDisplayName?.updateDisplayName() else null,
            publisher = if (arpPublisher != publisher) arpPublisher else null,
            displayVersion = if (displayVersion != sharedManifestData.packageVersion) displayVersion else null,
            upgradeCode = sharedManifestData.msi?.upgradeCode ?: upgradeCode
        )
    }

    private fun String.updateDisplayName(): String {
        return sharedManifestData.allVersions?.joinToString("|") { it }
            ?.let { replaceFirst(Regex(it), sharedManifestData.packageVersion) }
            ?: this
    }

    fun createInstallerManifest(): String {
        val installersLocaleDistinct = installers.mapNotNull { it.installerLocale }.distinct().size == 1
        val releaseDateDistinct = installers.mapNotNull { it.releaseDate }.distinct().size == 1
        val installerScopeDistinct = installers.mapNotNull { it.scope }.distinct().size == 1
        val upgradeBehaviourDistinct = installers.mapNotNull { it.upgradeBehavior }.distinct().size == 1
        val installerSwitchesDistinct = installers.mapNotNull { it.installerSwitches }.distinct().size == 1
        val installerTypeDistinct = installers.mapNotNull { it.installerType }.distinct().size == 1
        val platformDistinct = installers.mapNotNull { it.platform }.distinct().size == 1
        val minimumOSVersionDistinct = installers.mapNotNull { it.minimumOSVersion }.distinct().size == 1
        return getInstallerManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            installerLocale = when {
                installersLocaleDistinct -> installers.firstNotNullOf { it.installerLocale }.ifBlank { null }
                else -> previousManifestData.remoteInstallerData?.installerLocale
            },
            platform = when {
                platformDistinct -> installers.firstNotNullOf { it.platform }.map { it.toManifestPlatform() }
                else -> previousManifestData.remoteInstallerData?.platform
            },
            minimumOSVersion = when {
                minimumOSVersionDistinct -> installers.firstNotNullOf { it.minimumOSVersion }
                else -> previousManifestData.remoteInstallerData?.minimumOSVersion
            },
            installerType = when {
                installerTypeDistinct -> installers.firstNotNullOf { it.installerType }.toManifestInstallerType()
                else -> previousManifestData.remoteInstallerData?.installerType
            },
            scope = when {
                installerScopeDistinct -> installers.firstNotNullOf { it.scope }.toManifestScope()
                else -> previousManifestData.remoteInstallerData?.scope
            },
            installModes = installModes?.ifEmpty { null } ?: previousManifestData.remoteInstallerData?.installModes,
            installerSwitches = when {
                installerSwitchesDistinct -> {
                    installers.firstNotNullOf { it.installerSwitches }.toManifestInstallerSwitches()
                }
                else -> previousManifestData.remoteInstallerData?.installerSwitches
            },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: previousManifestData.remoteInstallerData?.installerSuccessCodes,
            upgradeBehavior = when {
                upgradeBehaviourDistinct -> installers.firstNotNullOf { it.upgradeBehavior }.toManifestUpgradeBehaviour()
                else -> previousManifestData.remoteInstallerData?.upgradeBehavior
            },
            commands = commands?.ifEmpty { null } ?: previousManifestData.remoteInstallerData?.commands,
            protocols = protocols?.ifEmpty { null } ?: previousManifestData.remoteInstallerData?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null }
                ?: previousManifestData.remoteInstallerData?.fileExtensions,
            releaseDate = if (releaseDateDistinct) installers.map { it.releaseDate }.first() else null,
            appsAndFeaturesEntries = when (installers.distinctBy { it.appsAndFeaturesEntries }.size) {
                0 -> previousManifestData.remoteInstallerData?.appsAndFeaturesEntries
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
            manifestVersion = schemasImpl.manifestOverride ?: Schemas.manifestVersion
        ).toString()
    }

    private fun getInstallerManifestBase(): InstallerManifest {
        return previousManifestData.remoteInstallerData ?: InstallerManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            manifestType = Schemas.installerManifestType,
            manifestVersion = schemasImpl.manifestOverride ?: Schemas.manifestVersion
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
