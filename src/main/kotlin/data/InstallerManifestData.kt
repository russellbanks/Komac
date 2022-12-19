package data

import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import kotlinx.serialization.modules.SerializersModule
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.LocalDateSerializer
import schemas.Schemas
import schemas.SchemasImpl
import schemas.TerminalInstance
import java.time.LocalDate

@Single
class InstallerManifestData : KoinComponent {
    lateinit var installerUrl: String
    lateinit var installerSha256: String
    lateinit var architecture: InstallerManifest.Installer.Architecture
    lateinit var installerType: InstallerManifest.InstallerType
    var silentSwitch: String? = null
    var silentWithProgressSwitch: String? = null
    var customSwitch: String? = null
    var installerLocale: String? = null
    var productCode: String? = null
    var installerScope: InstallerManifest.Scope? = null
    var upgradeBehavior: InstallerManifest.UpgradeBehavior? = null
    var releaseDate: LocalDate? = null
    private var installers = listOf<InstallerManifest.Installer>()
    var fileExtensions: List<String>? = null
    var protocols: List<String>? = null
    var commands: List<String>? = null
    var installerSuccessCodes: List<Int>? = null
    var installModes: List<InstallerManifest.InstallModes>? = null

    private val terminalInstance: TerminalInstance by inject()
    private val schemaImpl: SchemasImpl by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val installerSchema
        get() = schemaImpl.installerSchema

    fun addInstaller() {
        installers += InstallerManifest.Installer(
            installerLocale = installerLocale?.ifBlank { null },
            architecture = architecture,
            installerType = installerType,
            installerUrl = installerUrl,
            installerSha256 = installerSha256,
            scope = installerScope,
            installerSwitches = InstallerManifest.InstallerSwitches(
                silent = silentSwitch?.ifBlank { null },
                silentWithProgress = silentWithProgressSwitch?.ifBlank { null },
                custom = customSwitch?.ifBlank { null }
            ).takeUnless { it.areAllNull() },
            upgradeBehavior = upgradeBehavior,
            productCode = productCode?.ifBlank { null },
            releaseDate = releaseDate
        )
    }

    fun createInstallerManifest() {
        val installersLocaleDistinct = installers.distinctBy { it.installerLocale }.size == 1
        val releaseDateDistinct = installers.distinctBy { it.releaseDate }.size == 1
        val installerScopeDistinct = installers.distinctBy { it.scope }.size == 1
        val upgradeBehaviourDistinct = installers.distinctBy { it.upgradeBehavior }.size == 1
        val installerSwitchesDistinct = installers.distinctBy { it.installerSwitches }.size == 1
        val installerTypeDistinct = installers.distinctBy { it.installerType }.size == 1
        InstallerManifest(
            packageIdentifier = sharedManifestData.packageIdentifier,
            packageVersion = sharedManifestData.packageVersion,
            installerLocale = if (installersLocaleDistinct) installerLocale?.ifBlank { null } else null,
            installerType = if (installers.distinctBy { it.installerType }.size == 1) installerType else null,
            scope = if (installerScopeDistinct) installerScope else null,
            installModes = installModes?.ifEmpty { null },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null },
            upgradeBehavior = if (upgradeBehaviourDistinct) upgradeBehavior else null,
            commands = commands?.ifEmpty { null },
            protocols = protocols?.ifEmpty { null },
            fileExtensions = fileExtensions?.ifEmpty { null },
            releaseDate = if (releaseDateDistinct) releaseDate else null,
            installers = installers.map { installer ->
                installer.copy(
                    installerLocale = if (installersLocaleDistinct) null else installer.installerLocale,
                    scope = if (installerScopeDistinct) null else installer.scope,
                    releaseDate = if (releaseDateDistinct) null else installer.releaseDate,
                    upgradeBehavior = if (upgradeBehaviourDistinct) null else installer.upgradeBehavior,
                    installerSwitches = if (installerSwitchesDistinct) null else installer.installerSwitches,
                    installerType = if (installerTypeDistinct) null else installer.installerType,
                )
            },
            manifestType = Schemas.manifestType(installerSchema),
            manifestVersion = installerSchema.properties.manifestVersion.default
        ).also {
            Yaml(
                serializersModule = SerializersModule {
                    contextual(LocalDate::class, LocalDateSerializer)
                },
                configuration = YamlConfiguration(
                    encodeDefaults = false,
                    singleLineStringStyle = SingleLineStringStyle.Plain
                )
            ).run {
                buildString {
                    appendLine(Schemas.Comments.createdBy)
                    appendLine(Schemas.Comments.languageServer(installerSchema.id))
                    appendLine()
                    appendLine(encodeToString(InstallerManifest.serializer(), it))
                }.let(terminalInstance.terminal::print)
            }
        }
    }
}
