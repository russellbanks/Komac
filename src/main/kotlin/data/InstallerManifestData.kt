package data

import io.ktor.http.URLBuilder
import io.ktor.http.Url
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.Schema
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
    lateinit var installerType: InstallerManifest.Installer.InstallerType
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
    var installerSuccessCodes: List<Int>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null
    private val schemaImpl: SchemasImpl by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun addInstaller() {
        previousManifestData.remoteInstallerDataJob.join()
        val previousManifest = previousManifestData.remoteInstallerData
        val previousInstaller = previousManifest?.installers?.get(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = installerLocale?.ifBlank { null } ?: previousInstaller?.installerLocale,
            platform = sharedManifestData.msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform
                ?: previousManifest?.platform?.map { it.toPerInstallerPlatform() },
            minimumOSVersion = sharedManifestData.msix?.minVersion,
            architecture = if (::architecture.isInitialized) architecture else previousInstaller?.architecture!!,
            installerType = if (::installerType.isInitialized) installerType else previousInstaller?.installerType,
            nestedInstallerType = sharedManifestData.zip?.nestedInstallerType
                ?: previousInstaller?.nestedInstallerType
                ?: previousManifest?.nestedInstallerType?.toPerInstallerNestedInstallerType(),
            nestedInstallerFiles = sharedManifestData.zip?.nestedInstallerFiles
                .takeIf { it?.isNotEmpty() == true }
                ?: previousInstaller?.nestedInstallerFiles
                ?: previousManifest?.nestedInstallerFiles?.map { it.toPerInstallerNestedInstallerFiles() },
            installerUrl = installerUrl,
            installerSha256 = installerSha256.uppercase(),
            signatureSha256 = sharedManifestData.msix?.signatureSha256
                ?: sharedManifestData.msixBundle?.signatureSha256,
            scope = scope ?: previousInstaller?.scope ?: previousManifest?.scope?.toPerScopeInstallerType(),
            installerSwitches = installerSwitches
                .takeUnless { it.areAllNullOrBlank() }
                ?: previousInstaller?.installerSwitches
                ?: previousManifest?.installerSwitches?.toPerInstallerSwitches(),
            upgradeBehavior = upgradeBehavior?.toPerInstallerUpgradeBehaviour()
                ?: previousInstaller?.upgradeBehavior
                ?: previousManifest?.upgradeBehavior?.toPerInstallerUpgradeBehaviour(),
            productCode = sharedManifestData.msi?.productCode
                ?: productCode?.ifBlank { null }
                ?: previousManifest?.productCode,
            releaseDate = releaseDate ?: sharedManifestData.gitHubDetection?.releaseDate?.await(),
            appsAndFeaturesEntries = previousInstaller?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.fillARPEntry()
            } ?: previousManifest?.appsAndFeaturesEntries?.map { appsAndFeaturesEntry ->
                appsAndFeaturesEntry.toInstallerAppsAndFeaturesEntry().fillARPEntry()
            } ?: listOf(InstallerManifest.Installer.AppsAndFeaturesEntry().fillARPEntry()).ifEmpty { null },
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

    private fun InstallerManifest.Installer.AppsAndFeaturesEntry.fillARPEntry()
    : InstallerManifest.Installer.AppsAndFeaturesEntry {
        val arpDisplayName = sharedManifestData.msi?.productName ?: displayName
        val packageName = sharedManifestData.packageName ?: previousManifestData.remoteDefaultLocaleData?.packageName
        val arpPublisher = sharedManifestData.msi?.manufacturer ?: publisher
        val publisher = sharedManifestData.publisher ?: previousManifestData.remoteDefaultLocaleData?.publisher
        val displayVersion = sharedManifestData.msi?.productVersion ?: displayVersion
        return copy(
            displayName = if (arpDisplayName != packageName) arpDisplayName else null,
            publisher = if (arpPublisher != publisher) arpPublisher else null,
            displayVersion = if (displayVersion != sharedManifestData.packageVersion) displayVersion else null,
            upgradeCode = sharedManifestData.msi?.upgradeCode ?: upgradeCode
        )
    }

    private inline fun <T, R : Any> Iterable<T>.onlyOneNotNullDistinct(selector: (T) -> R?): Boolean {
        return mapNotNullTo(ArrayList(), selector).distinct().size == 1
    }

    fun createInstallerManifest(): String {
        val installersLocaleDistinct = installers.onlyOneNotNullDistinct { it.installerLocale }
        val releaseDateDistinct = installers.onlyOneNotNullDistinct { it.releaseDate }
        val installerScopeDistinct = installers.onlyOneNotNullDistinct { it.scope }
        val upgradeBehaviourDistinct = installers.onlyOneNotNullDistinct { it.upgradeBehavior }
        val installerSwitchesDistinct = installers.onlyOneNotNullDistinct { it.installerSwitches }
        val installerTypeDistinct = installers.onlyOneNotNullDistinct { it.installerType }
        val platformDistinct = installers.onlyOneNotNullDistinct { it.platform }
        val minimumOSVersionDistinct = installers.onlyOneNotNullDistinct { it.minimumOSVersion }
        val arpDistinct = installers.onlyOneNotNullDistinct { it.appsAndFeaturesEntries }
        return getInstallerManifestBase().copy(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            installerLocale = when {
                installersLocaleDistinct -> installers.map { it.installerLocale }.first()?.ifBlank { null }
                else -> previousManifestData.remoteInstallerData?.installerLocale
            },
            platform = when {
                platformDistinct -> installers.map { it.platform }.first()?.map { it.toManifestPlatform() }
                else -> previousManifestData.remoteInstallerData?.platform
            },
            minimumOSVersion = when {
                minimumOSVersionDistinct -> installers.map { it.minimumOSVersion }.first()
                else -> previousManifestData.remoteInstallerData?.minimumOSVersion
            },
            installerType = when {
                installerTypeDistinct -> installers.map { it.installerType }.first()?.toManifestInstallerType()
                else -> previousManifestData.remoteInstallerData?.installerType
            },
            scope = when {
                installerScopeDistinct -> installers.map { it.scope }.first()?.toManifestScope()
                else -> previousManifestData.remoteInstallerData?.scope
            },
            installModes = installModes
                .takeIf { it?.isNotEmpty() == true } ?: previousManifestData.remoteInstallerData?.installModes,
            installerSwitches = when {
                installerSwitchesDistinct -> {
                    installers.map { it.installerSwitches }.first()?.toManifestInstallerSwitches()
                }
                else -> previousManifestData.remoteInstallerData?.installerSwitches
            },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null }
                ?: previousManifestData.remoteInstallerData?.installerSuccessCodes,
            upgradeBehavior = when {
                upgradeBehaviourDistinct -> installers.map { it.upgradeBehavior }.first()?.toManifestUpgradeBehaviour()
                else -> previousManifestData.remoteInstallerData?.upgradeBehavior
            },
            commands = commands?.ifEmpty { null } ?: previousManifestData.remoteInstallerData?.commands,
            protocols = protocols?.ifEmpty { null } ?: previousManifestData.remoteInstallerData?.protocols,
            fileExtensions = fileExtensions?.ifEmpty { null }
                ?: previousManifestData.remoteInstallerData?.fileExtensions,
            releaseDate = when {
                releaseDateDistinct -> installers.map { it.releaseDate }.first()
                else -> null
            },
            appsAndFeaturesEntries = when {
                arpDistinct -> {
                    installers
                        .first()
                        .appsAndFeaturesEntries
                        ?.map { it.toManifestARPEntry() }
                }
                else -> previousManifestData.remoteInstallerData?.appsAndFeaturesEntries
            },
            installers = installers.removeNonDistinctKeys()
                .sortedWith(compareBy({ it.installerLocale }, { it.installerType }, { it.architecture }, { it.scope })),
            manifestType = schemaImpl.installerSchema.properties.manifestType.const,
            manifestVersion = schemaImpl.installerSchema.properties.manifestVersion.default
        ).toEncodedYaml()
    }

    private fun getInstallerManifestBase(): InstallerManifest {
        return previousManifestData.remoteInstallerData ?: InstallerManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            manifestType = Schemas.manifestType(schemaImpl.installerSchema),
            manifestVersion = schemaImpl.installerSchema.properties.manifestVersion.default
        )
    }

    private fun List<InstallerManifest.Installer>.removeNonDistinctKeys(): List<InstallerManifest.Installer> {
        return map { installer ->
            installer.copy(
                installerLocale = if (onlyOneNotNullDistinct { it.installerLocale }) {
                    null
                } else {
                    installer.installerLocale
                },
                platform = if (onlyOneNotNullDistinct { it.platform }) null else installer.platform,
                minimumOSVersion = if (onlyOneNotNullDistinct { it.minimumOSVersion }) {
                    null
                } else {
                    installer.minimumOSVersion
                },
                installerType = if (onlyOneNotNullDistinct { it.installerType }) null else installer.installerType,
                scope = if (onlyOneNotNullDistinct { it.scope }) null else installer.scope,
                releaseDate = if (onlyOneNotNullDistinct { it.releaseDate }) null else installer.releaseDate,
                upgradeBehavior = if (onlyOneNotNullDistinct { it.upgradeBehavior }) {
                    null
                } else {
                    installer.upgradeBehavior
                },
                installerSwitches = if (onlyOneNotNullDistinct { it.installerSwitches }) {
                    null
                } else {
                    installer.installerSwitches
                },
                appsAndFeaturesEntries = if (onlyOneNotNullDistinct { it.appsAndFeaturesEntries }) {
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

    private fun InstallerManifest.toEncodedYaml(): String {
        return Schemas.buildManifestString(
            schema = Schema.Installer,
            rawString = YamlConfig.default.encodeToString(
                serializer = InstallerManifest.serializer(),
                value = this@toEncodedYaml
            )
        )
    }

    private fun resetValues() {
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
        sharedManifestData.gitHubDetection?.releaseDate = null
    }
}
