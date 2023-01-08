package data

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.Schemas
import schemas.SchemasImpl
import java.time.LocalDate

@Single
class InstallerManifestData : KoinComponent {
    lateinit var installerUrl: String
    lateinit var installerSha256: String
    lateinit var architecture: InstallerManifest.Installer.Architecture
    lateinit var installerType: InstallerManifest.InstallerType
    var signatureSha256: String? = null
    var silentSwitch: String? = null
    var silentWithProgressSwitch: String? = null
    var customSwitch: String? = null
    var installerLocale: String? = null
    var productCode: String? = null
    var scope: InstallerManifest.Scope? = null
    var upgradeBehavior: InstallerManifest.UpgradeBehavior? = null
    var releaseDate: LocalDate? = null
    var installers = listOf<InstallerManifest.Installer>()
    var fileExtensions: List<String>? = null
    var protocols: List<String>? = null
    var commands: List<String>? = null
    var installerSuccessCodes: List<Int>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null
    private val schemaImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val installerSchema
        get() = schemaImpl.installerSchema

    fun createInstaller(): InstallerManifest.Installer {
        return InstallerManifest.Installer(
            installerLocale = installerLocale?.ifBlank { null },
            architecture = architecture,
            installerType = installerType.toPerInstallerType(),
            installerUrl = installerUrl,
            installerSha256 = installerSha256.uppercase(),
            signatureSha256 = signatureSha256?.uppercase()?.ifBlank { null },
            scope = scope?.toPerScopeInstallerType(),
            installerSwitches = InstallerManifest.Installer.InstallerSwitches(
                silent = silentSwitch?.ifBlank { null },
                silentWithProgress = silentWithProgressSwitch?.ifBlank { null },
                custom = customSwitch?.ifBlank { null }
            ).takeUnless { it.areAllNull() },
            upgradeBehavior = upgradeBehavior?.toPerInstallerType(),
            productCode = productCode?.ifBlank { null },
            releaseDate = releaseDate
        )
    }

    fun createInstallerManifest(): String {
        val installersLocaleDistinct = installers.distinctBy { it.installerLocale }.size == 1
        val releaseDateDistinct = installers.distinctBy { it.releaseDate }.size == 1
        val installerScopeDistinct = installers.distinctBy { it.scope }.size == 1
        val upgradeBehaviourDistinct = installers.distinctBy { it.upgradeBehavior }.size == 1
        val installerSwitchesDistinct = installers.distinctBy { it.installerSwitches }.size == 1
        val installerTypeDistinct = installers.distinctBy { it.installerType }.size == 1
        val platformDistinct = installers.distinctBy { it.platform }.size == 1
        val minimumOSVersionDistinct = installers.distinctBy { it.minimumOSVersion }.size == 1
        return InstallerManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            installerLocale = when {
                installersLocaleDistinct -> installers.map { it.installerLocale }.first()?.ifBlank { null }
                else -> null
            },
            platform = when {
                platformDistinct -> installers.map { it.platform }.first()?.map { it.toManifestPlatform() }
                else -> null
            },
            minimumOSVersion = if (minimumOSVersionDistinct) installers.map { it.minimumOSVersion }.first() else null,
            installerType = if (installers.distinctBy { it.installerType }.size == 1) installerType else null,
            scope = if (installerScopeDistinct) installers.map { it.scope }.first()?.toManifestScope() else null,
            installModes = installModes?.ifEmpty { null },
            installerSwitches = when {
                installerSwitchesDistinct -> {
                    installers.map { it.installerSwitches }.first()?.toManifestInstallerSwitches()
                }
                else -> null
            },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null },
            upgradeBehavior = when {
                upgradeBehaviourDistinct -> installers.map { it.upgradeBehavior }.first()?.toManifestUpgradeBehaviour()
                else -> null
            },
            commands = commands?.ifEmpty { null },
            protocols = protocols?.ifEmpty { null },
            fileExtensions = fileExtensions?.ifEmpty { null },
            releaseDate = if (releaseDateDistinct) installers.map { it.releaseDate }.first() else null,
            installers = installers.map { installer ->
                installer.copy(
                    platform = if (platformDistinct) null else installer.platform,
                    minimumOSVersion = if (minimumOSVersionDistinct) null else installer.minimumOSVersion,
                    installerLocale = if (installersLocaleDistinct) null else installer.installerLocale,
                    scope = if (installerScopeDistinct) null else installer.scope,
                    releaseDate = if (releaseDateDistinct) null else installer.releaseDate,
                    upgradeBehavior = if (upgradeBehaviourDistinct) null else installer.upgradeBehavior,
                    installerSwitches = if (installerSwitchesDistinct) null else installer.installerSwitches,
                    installerType = if (installerTypeDistinct) null else installer.installerType,
                )
            }.sortedWith(compareBy({ it.installerLocale }, { it.installerType }, { it.architecture }, { it.scope })),
            manifestType = Schemas.manifestType(installerSchema),
            manifestVersion = installerSchema.properties.manifestVersion.default
        ).let {
            get<GitHubImpl>().buildManifestString(get<SchemasImpl>().installerSchema.id) {
                appendLine(
                    YamlConfig.defaultWithLocalDataSerializer.encodeToString(InstallerManifest.serializer(), it)
                )
            }
        }
    }
}
