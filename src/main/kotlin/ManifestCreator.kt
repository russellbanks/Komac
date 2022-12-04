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
        val defaultEncoder = Yaml(configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain
        ))
        val yamlOutput = defaultEncoder.encodeToString(InstallerManifest.serializer(), manifest)
        val createdByComment = "${Schemas.Comments.createdBy}\n"
        val languageServerComment = "${Schemas.Comments.installerLanguageServer}\n"
        terminalInstance.terminal.println("$createdByComment$languageServerComment\n$yamlOutput")
    }
}