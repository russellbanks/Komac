import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.Schemas
import schemas.TerminalInstance

class ManifestCreator : KoinComponent {
    private val terminalInstance: TerminalInstance by inject()

    fun createInstallerManifest(
        packageIdentifier: String,
        packageVersion: String,
        architecture: String,
        installerUrl: String,
        installerSha256: String
    ) {
        val manifest = InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            installers = listOf(
                InstallerManifest.Installer(
                    architecture = architecture,
                    installerUrl = installerUrl,
                    installerSha256 = installerSha256
                )
            ),
            manifestVersion = Schemas.manifestVersion
        )
        val yamlEncoder = Yaml(configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain
        ))
        buildString {
            appendLine(Schemas.Comments.createdBy)
            appendLine(Schemas.Comments.installerLanguageServer)
            appendLine()
            appendLine(yamlEncoder.encodeToString(InstallerManifest.serializer(), manifest))
        }.let(terminalInstance.terminal::print)
    }
}