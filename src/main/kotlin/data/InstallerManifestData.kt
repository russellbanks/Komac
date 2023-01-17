package data

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
    lateinit var installerUrl: String
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
        val previousInstaller = previousManifestData.remoteInstallerData?.installers?.get(installers.size)
        val installer = getInstallerBase(previousInstaller).copy(
            installerLocale = installerLocale?.ifBlank { null } ?: previousInstaller?.installerLocale,
            platform = sharedManifestData.msix?.targetDeviceFamily?.let { listOf(it.toPerInstallerPlatform()) }
                ?: previousInstaller?.platform,
            minimumOSVersion = sharedManifestData.msix?.minVersion,
            architecture = if (::architecture.isInitialized) architecture else previousInstaller?.architecture!!,
            installerType = if (::installerType.isInitialized) installerType else previousInstaller?.installerType,
            nestedInstallerType = sharedManifestData.zip?.nestedInstallerType ?: previousInstaller?.nestedInstallerType,
            nestedInstallerFiles = sharedManifestData.zip?.nestedInstallerFiles
                .takeIf { it?.isNotEmpty() == true } ?: previousInstaller?.nestedInstallerFiles,
            installerUrl = installerUrl,
            installerSha256 = installerSha256.uppercase(),
            signatureSha256 = when {
                sharedManifestData.msix?.signatureSha256 != null -> sharedManifestData.msix?.signatureSha256
                sharedManifestData.msixBundle?.signatureSha256 != null -> sharedManifestData.msixBundle?.signatureSha256
                else -> null
            },
            scope = scope ?: previousInstaller?.scope,
            installerSwitches = installerSwitches
                .takeUnless { it.areAllNullOrBlank() } ?: previousInstaller?.installerSwitches,
            upgradeBehavior = upgradeBehavior?.toPerInstallerUpgradeBehaviour() ?: previousInstaller?.upgradeBehavior,
            productCode = sharedManifestData.msi?.productCode ?: productCode?.ifBlank { null },
            releaseDate = releaseDate ?: sharedManifestData.gitHubDetection?.releaseDate?.await(),
            appsAndFeaturesEntries = previousInstaller?.appsAndFeaturesEntries?.map {
                it.copy(upgradeCode = sharedManifestData.msi?.upgradeCode)
            } ?: sharedManifestData.msi?.upgradeCode?.let {
                listOf(InstallerManifest.Installer.AppsAndFeaturesEntry(upgradeCode = it))
            },
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

    private inline fun <T, R : Any> Iterable<T>.onlyOneNotNullDistinct(selector: (T) -> R?): Boolean {
        return mapNotNullTo(ArrayList(), selector).distinct().size == 1
    }

    suspend fun createInstallerManifest(): String {
        val installersLocaleDistinct = installers.onlyOneNotNullDistinct { it.installerLocale }
        val releaseDateDistinct = installers.onlyOneNotNullDistinct { it.releaseDate }
        val installerScopeDistinct = installers.onlyOneNotNullDistinct { it.scope }
        val upgradeBehaviourDistinct = installers.onlyOneNotNullDistinct { it.upgradeBehavior }
        val installerSwitchesDistinct = installers.onlyOneNotNullDistinct { it.installerSwitches }
        val installerTypeDistinct = installers.onlyOneNotNullDistinct { it.installerType }
        val nestedInstallerTypeDistinct = installers.onlyOneNotNullDistinct { it.nestedInstallerType }
        val nestedInstallerFilesDistinct = installers.onlyOneNotNullDistinct { it.nestedInstallerFiles }
        val platformDistinct = installers.onlyOneNotNullDistinct { it.platform }
        val minimumOSVersionDistinct = installers.onlyOneNotNullDistinct { it.minimumOSVersion }
        val arpDistinct = installers.onlyOneNotNullDistinct { it.appsAndFeaturesEntries }
        previousManifestData.remoteInstallerDataJob.join()
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
            nestedInstallerType = when {
                nestedInstallerTypeDistinct -> {
                    installers.map { it.nestedInstallerType }.first()?.toManifestNestedInstallerType()
                }
                else -> previousManifestData.remoteInstallerData?.nestedInstallerType
            },
            nestedInstallerFiles = when {
                nestedInstallerFilesDistinct -> {
                    installers.map { it.nestedInstallerFiles }
                        .first()?.map { it.toManifestNestedInstallerFiles() }?.sortedBy { it.relativeFilePath }
                }
                else -> previousManifestData.remoteInstallerData?.nestedInstallerFiles
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
            installerSuccessCodes = installerSuccessCodes?.ifEmpty {
                previousManifestData.remoteInstallerData?.installerSuccessCodes
            },
            upgradeBehavior = when {
                upgradeBehaviourDistinct -> installers.map { it.upgradeBehavior }.first()?.toManifestUpgradeBehaviour()
                else -> previousManifestData.remoteInstallerData?.upgradeBehavior
            },
            commands = commands?.ifEmpty { previousManifestData.remoteInstallerData?.commands },
            protocols = protocols?.ifEmpty { previousManifestData.remoteInstallerData?.protocols },
            fileExtensions = fileExtensions?.ifEmpty { previousManifestData.remoteInstallerData?.fileExtensions },
            releaseDate = when {
                releaseDateDistinct -> installers.map { it.releaseDate }.first()
                else -> null
            },
            appsAndFeaturesEntries = when {
                arpDistinct -> {
                    installers
                        .first()
                        .appsAndFeaturesEntries
                        ?.map { appsAndFeatureEntry ->
                            sharedManifestData.msi?.productCode?.let {
                                when {
                                    sharedManifestData.packageName != it -> appsAndFeatureEntry.copy(displayName = it)
                                    else -> appsAndFeatureEntry
                                }
                            } ?: appsAndFeatureEntry
                        }
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
                installerLocale = if (onlyOneNotNullDistinct {
                        it.installerLocale
                }) null else installer.installerLocale,
                platform = if (onlyOneNotNullDistinct { it.platform }) null else installer.platform,
                minimumOSVersion = if (onlyOneNotNullDistinct { it.minimumOSVersion }) {
                    null
                } else {
                    installer.minimumOSVersion
                },
                installerType = if (onlyOneNotNullDistinct { it.installerType }) null else installer.installerType,
                nestedInstallerType = if (onlyOneNotNullDistinct { it.nestedInstallerType }) {
                    null
                } else {
                    installer.nestedInstallerType
                },
                nestedInstallerFiles = if (onlyOneNotNullDistinct { it.nestedInstallerFiles }) {
                    null
                } else {
                    installer.nestedInstallerFiles
                },
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
            installerUrl = ""
        )
    }

    private fun InstallerManifest.toEncodedYaml(): String {
        return Schemas.buildManifestString(
            schema = Schema.Installer,
            rawString = YamlConfig.defaultWithLocalDataSerializer.encodeToString(
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
