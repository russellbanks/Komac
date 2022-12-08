package data
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.Schemas
import schemas.TerminalInstance

@Single
class InstallerManifestData : KoinComponent {
    var packageVersion: String? = null
    var installerUrl: String? = null
    var packageIdentifier: String? = null
    var installerSha256: String? = null
    var architecture: String? = null
    var installerType: String? = null
    var silentSwitch: String? = null
    var silentWithProgressSwitch: String? = null
    var customSwitch: String? = null
    var installerLocale: String? = null
    var productCode: String? = null
    var installerScope: String? = null
    var upgradeBehavior: String? = null

    private val terminalInstance: TerminalInstance by inject()

    fun createInstallerManifest() {
        InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            installers = listOf(
                InstallerManifest.Installer(
                    architecture = architecture,
                    installerLocale = installerLocale?.ifBlank { null },
                    installerType = installerType,
                    installerUrl = installerUrl,
                    installerSha256 = installerSha256,
                    scope = installerScope?.ifBlank { null },
                    installerSwitches = InstallerManifest.Installer.InstallerSwitches(
                        silent = silentSwitch?.ifBlank { null },
                        silentWithProgress = silentWithProgressSwitch?.ifBlank { null },
                        custom = customSwitch?.ifBlank { null }
                    ),
                    upgradeBehavior = upgradeBehavior?.ifBlank { null },
                    productCode = productCode?.ifBlank { null }
                )
            ),
            manifestVersion = Schemas.manifestVersion
        ).also {
            Yaml(
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
