import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.InstallerManifestData
import data.SharedManifestData
import data.VersionManifestData
import data.YamlConfig
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import input.Prompts
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.Schemas
import schemas.SchemasImpl
import schemas.VersionManifest

class QuickUpdate(private val terminal: Terminal) : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocalManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    var installers = listOf<InstallerManifest.Installer>()

    suspend fun main() = coroutineScope {
        with(terminal) {
            packageIdentifierPrompt()
            launch { sharedManifestData.getPreviousManifestData() }
            launch {
                packageVersionPrompt()
                do {
                    sharedManifestData.remoteInstallerData.await()?.installers?.forEachIndexed { index, installer ->
                        println(
                            verticalLayout {
                                cell(brightGreen("Installer Entry #${index.inc()}"))
                                cell(
                                    brightYellow(
                                        buildString {
                                            append(" ".repeat(Prompts.optionIndent))
                                            append("Architecture: ${installer.architecture}")
                                        }
                                    )
                                )
                                installer.installerType?.let {
                                    cell(
                                        brightYellow(
                                            buildString {
                                                append(" ".repeat(Prompts.optionIndent))
                                                append("Installer Type ${installer.installerType}")
                                            }
                                        )
                                    )
                                }
                                installer.scope?.let {
                                    cell(
                                        brightYellow(
                                            buildString {
                                                append(" ".repeat(Prompts.optionIndent))
                                                append("Scope ${installer.scope}")
                                            }
                                        )
                                    )
                                }
                                installer.installerLocale?.let {
                                    cell(
                                        brightYellow(
                                            buildString {
                                                append(" ".repeat(Prompts.optionIndent))
                                                append("Locale ${installer.installerLocale}")
                                            }
                                        )
                                    )
                                }
                                cell("")
                            }
                        )
                        installerDownloadPrompt()
                        installers += installer.copy(
                            installerUrl = installerManifestData.installerUrl,
                            installerSha256 = installerManifestData.installerSha256,
                        )
                    }
                } while (
                    (sharedManifestData.remoteInstallerData.await()?.installers?.size ?: 0) <
                    installerManifestData.installers.size
                )
                YamlConfig.installer.encodeToString(
                    InstallerManifest.serializer(),
                    sharedManifestData.remoteInstallerData.await()!!.copy(
                        installers = installers,
                        manifestVersion = "1.4.0"
                    )
                ).let {
                    Schemas.manifestBuilder(get<SchemasImpl>().installerSchema.id) {
                        appendLine(it)
                    }.let(terminal::print)
                }
                YamlConfig.other.encodeToString(
                    DefaultLocaleManifest.serializer(),
                    sharedManifestData.remoteDefaultLocaleData.await()!!.copy(
                        manifestVersion = "1.4.0"
                    )
                ).let {
                    Schemas.manifestBuilder(get<SchemasImpl>().defaultLocaleSchema.id) {
                        appendLine(it)
                    }.let(terminal::print)
                }
                YamlConfig.other.encodeToString(
                    VersionManifest.serializer(),
                    sharedManifestData.remoteVersionData.await()!!.copy(
                        manifestVersion = "1.4.0"
                    )
                ).let {
                    Schemas.manifestBuilder(get<SchemasImpl>().versionSchema.id) {
                        appendLine(it)
                    }.let(terminal::print)
                }
            }
        }
    }
}
