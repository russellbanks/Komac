package data
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import kotlinx.serialization.modules.SerializersModule
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchemaImpl
import schemas.LocalDateSerializer
import schemas.Schemas
import schemas.TerminalInstance
import java.time.LocalDate

@Single
class InstallerManifestData : KoinComponent {
    lateinit var packageIdentifier: String
    lateinit var packageVersion: String
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

    private val terminalInstance: TerminalInstance by inject()
    private val installerSchemaImpl: InstallerSchemaImpl by inject()
    private val installerSchema
        get() = installerSchemaImpl.installerSchema

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
        InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            commands = commands?.ifEmpty { null },
            installerSuccessCodes = installerSuccessCodes?.ifEmpty { null },
            fileExtensions = fileExtensions?.ifEmpty { null },
            protocols = protocols?.ifEmpty { null },
            installers = installers,
            manifestType = Schemas.manifestType(installerSchema),
            manifestVersion = Schemas.manifestVersion
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
                    appendLine(Schemas.Comments.installerLanguageServer)
                    appendLine()
                    appendLine(encodeToString(InstallerManifest.serializer(), it))
                }.let(terminalInstance.terminal::print)
            }
        }
    }
}
