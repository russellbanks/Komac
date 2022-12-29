import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.optional
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import data.GitHubImpl
import data.InstallerManifestData
import data.SharedManifestData
import data.YamlConfig
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import input.PromptType
import input.Prompts
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.kohsuke.github.GHRepository
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.ManifestBuilder
import schemas.SchemasImpl
import schemas.TerminalInstance
import schemas.VersionManifest
import java.io.FileWriter
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val urls: List<String>? by argument("--urls").multiple().optional()

    private val installerManifestData: InstallerManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    var installers = listOf<InstallerManifest.Installer>()

    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            packageIdentifierPrompt()
            launch { sharedManifestData.getPreviousManifestData() }
            launch {
                packageVersionPrompt()
                do {
                    sharedManifestData.remoteInstallerData?.installers?.forEachIndexed { index, installer ->
                        println(
                            verticalLayout {
                                cell(brightGreen("Installer Entry #${index.inc()}"))
                                listOf(
                                    PromptType.Architecture to installer.architecture,
                                    PromptType.InstallerType to installer.installerType,
                                    PromptType.Scope to installer.scope,
                                    PromptType.InstallerLocale to installer.installerLocale
                                ).forEach { (promptType, value) ->
                                    if (value != null) {
                                        cell(brightYellow("${" ".repeat(Prompts.optionIndent)} $promptType: $value"))
                                    }
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
                    (sharedManifestData.remoteInstallerData?.installers?.size ?: 0) <
                    installerManifestData.installers.size
                )
                sharedManifestData.defaultLocale = sharedManifestData.remoteVersionData!!.defaultLocale
                val installerManifest = YamlConfig.installer.encodeToString(
                    InstallerManifest.serializer(),
                    sharedManifestData.remoteInstallerData!!.copy(
                        packageIdentifier = sharedManifestData.packageIdentifier,
                        packageVersion = sharedManifestData.packageVersion,
                        installers = installers,
                        manifestVersion = "1.4.0"
                    )
                ).let {
                    ManifestBuilder.buildManifestString(get<SchemasImpl>().installerSchema.id) {
                        appendLine(it)
                    }.also { println(it) }
                }
                val defaultLocaleManifest = YamlConfig.other.encodeToString(
                    DefaultLocaleManifest.serializer(),
                    sharedManifestData.remoteDefaultLocaleData!!.copy(
                        packageIdentifier = sharedManifestData.packageIdentifier,
                        packageVersion = sharedManifestData.packageVersion,
                        manifestVersion = "1.4.0"
                    )
                ).let {
                    ManifestBuilder.buildManifestString(get<SchemasImpl>().defaultLocaleSchema.id) {
                        appendLine(it)
                    }.also { println(it) }
                }
                val versionManifest = sharedManifestData.remoteVersionData?.copy(
                    packageIdentifier = sharedManifestData.packageIdentifier,
                    packageVersion = sharedManifestData.packageVersion,
                    manifestVersion = "1.4.0"
                )?.let { versionManifest ->
                    YamlConfig.other.encodeToString(VersionManifest.serializer(), versionManifest).let {
                        ManifestBuilder.buildManifestString(get<SchemasImpl>().versionSchema.id) {
                            appendLine(it)
                        }.also { println(it) }
                    }
                }
                val directories: List<String> = listOf(
                    komacTemp,
                    sharedManifestData.packageIdentifier.first().toString().lowercase()
                ) + sharedManifestData.packageIdentifier.split(".") + listOf(
                    sharedManifestData.packageVersion
                )
                val tempDirectory: Path = Paths.get(System.getProperty("java.io.tmpdir"))
                var parent: Path = tempDirectory
                directories.forEach { directory ->
                    parent = parent.resolve(directory)
                    if (!Files.exists(parent)) {
                        Files.createDirectory(parent).also {
                            println("Created directory $it")
                        }
                    }
                }
                withContext(Dispatchers.IO) {
                    FileWriter(parent.resolve(ManifestBuilder.installerManifestName).toFile()).use { fileWriter ->
                        fileWriter.write(installerManifest.replace("\n", "\r\n"))
                    }
                    FileWriter(parent.resolve(ManifestBuilder.defaultLocaleManifestName).toFile()).use { fileWriter ->
                        fileWriter.write(defaultLocaleManifest.replace("\n", "\r\n"))
                    }
                    FileWriter(parent.resolve(ManifestBuilder.versionManifestName).toFile()).use { fileWriter ->
                        versionManifest?.replace("\n", "\r\n")?.let { fileWriter.write(it) }
                    }
                }
                val github = get<GitHubImpl>().github
                val username = github.myself.login
                val repository: GHRepository = try {
                    github.getRepository("$username/winget-pkgs")
                } catch (_: IOException) {
                    github.getRepository("Microsoft/winget-pkgs").fork()
                }
                val ref = repository.createRef(
                    "refs/heads/${ManifestBuilder.branchName}",
                    repository.getBranch(repository.defaultBranch).shA1
                )
                val commit = repository.createCommit()
                    .message("This is a test commit for Komac")
                    .parent(ref.getObject().sha)
                    .tree(
                        repository
                            .createTree()
                            .baseTree(repository.getBranch(ref.ref).shA1)
                            .add(
                                ManifestBuilder.installerManifestGitHubPath,
                                parent.resolve(ManifestBuilder.installerManifestName).toFile().readBytes(),
                                false
                            )
                            .add(
                                ManifestBuilder.defaultLocaleManifestGitHubPath,
                                parent.resolve(ManifestBuilder.defaultLocaleManifestName).toFile().readBytes(),
                                false
                            )
                            .add(
                                ManifestBuilder.versionManifestGitHubPath,
                                parent.resolve(ManifestBuilder.versionManifestName).toFile().readBytes(),
                                false
                            )
                            .create()
                            .sha
                    )
                    .create()

                ref.updateTo(commit.shA1)

                tempDirectory.resolve(komacTemp).toFile().deleteRecursively()
            }
        }
    }

    companion object {
        const val komacTemp = "komac-tmp"
    }
}
