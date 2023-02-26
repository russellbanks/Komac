package commands

import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.clikt.parameters.options.validate
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandUtils.prompt
import data.AllManifestData
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.ManifestUtils.formattedManifestLinesFlow
import data.PreviousManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.installer.InstallerType
import data.shared.Locale
import data.shared.PackageIdentifier
import data.shared.PackageIdentifier.getLatestVersion
import data.shared.PackageVersion
import data.shared.Url.installerDownloadPrompt
import detection.ParameterUrls
import detection.github.GitHubDetection
import input.FileWriter
import input.ManifestResultOption
import input.Prompts
import input.Prompts.pullRequestPrompt
import io.ktor.client.call.body
import io.ktor.client.request.prepareGet
import io.ktor.http.Url
import io.ktor.http.contentLength
import io.ktor.http.lastModified
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.core.isNotEmpty
import io.ktor.utils.io.core.readBytes
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils
import network.findArchitecture
import network.findScope
import org.kohsuke.github.GitHub
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.AdditionalMetadata
import schemas.Schema
import schemas.Schemas
import schemas.manifest.EncodeConfig
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import token.Token
import token.TokenStore
import utils.FileAnalyser
import utils.FileUtils
import utils.Hashing.hash
import java.io.File
import java.time.LocalDate
import java.time.ZoneOffset

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val allManifestData: AllManifestData by inject()
    private val tokenStore: TokenStore by inject()
    private lateinit var previousManifestData: PreviousManifestData
    private val githubImpl: GitHubImpl by inject()
    private val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    private val packageIdentifierParam: String? by option("--id", "--package-identifier")
    private val packageVersionParam: String? by option("--version", "--package-version")
    private val urls: List<Url>? by option().convert { Url(it) }.split(",")
    private val manifestVersion: String? by option().validate {
        require(Regex("^\\d+\\.\\d+\\.\\d+$").matches(it)) { "Manifest version must be in the format X.X.X" }
    }
    private val submit: Boolean by option().flag(default = false)
    private val tokenParameter: String? by option("-t", "--token", envvar = "GITHUB_TOKEN").validate {
        require(GitHub.connectUsingOAuth(it).isCredentialValid) {
            currentContext.terminal.colors.danger("The token is invalid or has expired")
        }
    }
    private val additionalMetadataParam by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }

    override fun run(): Unit = runBlocking {
        manifestVersion?.let { get<Schemas>().manifestOverride = it }
        tokenParameter?.let { tokenStore.useTokenParameter(it) }
        with(currentContext.terminal) {
            with(allManifestData) {
                if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
                if (isCIEnvironment) {
                    info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
                }
                packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
                if (!tokenStore.isTokenValid.await()) {
                    tokenStore.invalidTokenPrompt(currentContext.terminal)
                    echo()
                }
                latestVersion = getLatestVersion(packageIdentifier, !isCIEnvironment)
                previousManifestData = get()
                if (updateState == VersionUpdateState.NewPackage) {
                    throw CliktError(
                        colors.warning(
                            buildString {
                                appendLine("$packageIdentifier does not exist in ${GitHubImpl.wingetpkgs}.")
                                appendLine("Please use the 'new' command to create a new manifest.")
                            }
                        ),
                        statusCode = 1
                    )
                }
                packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
                githubImpl.promptIfPullRequestExists(
                    identifier = packageIdentifier,
                    version = packageVersion,
                    terminal = currentContext.terminal
                )
                PackageVersion.setUpgradeState(allManifestData)
                loopThroughInstallers(parameterUrls = urls, isCIEnvironment = isCIEnvironment)
                val files = createFiles()
                files.forEach { manifest ->
                    formattedManifestLinesFlow(manifest.second, colors).collect { echo(it) }
                }
                if (submit) {
                    githubImpl.commitAndPullRequest(files, currentContext.terminal)
                } else if (!isCIEnvironment) {
                    pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                        when (manifestResultOption) {
                            ManifestResultOption.PullRequest -> {
                                githubImpl.commitAndPullRequest(files, currentContext.terminal)
                            }
                            ManifestResultOption.WriteToFiles -> FileWriter.writeFiles(files, currentContext.terminal)
                            else -> return@also
                        }
                    }
                } else {
                    FileWriter.writeFilesToDirectory(
                        directory = File(System.getProperty("user.dir"), "$packageIdentifier version $packageVersion"),
                        files = files,
                        terminal = currentContext.terminal
                    )
                }
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: List<Url>? = null,
        isCIEnvironment: Boolean = false
    ) = with(allManifestData) {
        val remoteInstallerManifest = previousManifestData.remoteInstallerData.await()!!
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls = parameterUrls, remoteInstallerManifest = remoteInstallerManifest)
        } else if (isCIEnvironment) {
            throw CliktError(colors.danger("${Errors.error} No installers have been provided"), statusCode = 1)
        } else {
            remoteInstallerManifest.installers.forEachIndexed { index, installer ->
                info("Installer Entry ${index.inc()}/${remoteInstallerManifest.installers.size}")
                listOf(
                    InstallerManifest.Installer.Architecture::class.simpleName!! to installer.architecture,
                    InstallerType.const to (installer.installerType ?: remoteInstallerManifest.installerType),
                    InstallerManifest.Scope::class.simpleName!! to (installer.scope ?: remoteInstallerManifest.scope),
                    Locale.installerLocaleConst to
                        (installer.installerLocale ?: remoteInstallerManifest.installerLocale)
                ).forEach { (promptType, value) ->
                    value?.let {
                        echo("${" ".repeat(Prompts.optionIndent)} $promptType: ${colors.info(it.toString())}")
                    }
                }
                echo()
                installerDownloadPrompt()
                if (!skipAddInstaller) InstallerManifestData.addInstaller() else skipAddInstaller = false
            }
        }
    }

    private suspend fun Terminal.loopParameterUrls(
        parameterUrls: List<Url>,
        remoteInstallerManifest: InstallerManifest
    ) = with(allManifestData) {
        val previousInstallers = remoteInstallerManifest.installers
        val previousUrls = previousInstallers.map { it.installerUrl }
        ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls, this@loopParameterUrls)
        ParameterUrls.assertUrlsValid(parameterUrls, this@loopParameterUrls)
        val downloadTasks = mutableListOf<InstallerManifest.Installer>()
        val client = get<Http>().client
        parameterUrls.distinct().forEach { url ->
            val file = FileUtils.createTempFile(identifier = packageIdentifier, version = packageVersion, url = url)
            val progress = HttpUtils.getDownloadProgressBar(url, this@loopParameterUrls).also { it.start() }
            gitHubDetection = parameterUrls
                .firstOrNull { it.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true) }
                ?.let { GitHubDetection(it) }
            var releaseDate: LocalDate? = null
            client.prepareGet(url).execute { httpResponse ->
                releaseDate = httpResponse.lastModified()?.toInstant()?.atZone(ZoneOffset.UTC)?.toLocalDate()
                val channel: ByteReadChannel = httpResponse.body()
                while (!channel.isClosedForRead) {
                    val packet = channel.readRemaining(DEFAULT_BUFFER_SIZE.toLong())
                    while (packet.isNotEmpty) {
                        file.appendBytes(packet.readBytes())
                        progress.update(file.length(), httpResponse.contentLength())
                    }
                }
            }
            val fileAnalyser = FileAnalyser(file)
            downloadTasks += try {
                InstallerManifest.Installer(
                    architecture = url.findArchitecture() ?: fileAnalyser.getArchitecture(),
                    installerType = fileAnalyser.getInstallerType(),
                    scope = url.findScope(),
                    installerSha256 = file.hash(),
                    signatureSha256 = fileAnalyser.getSignatureSha256(),
                    installerUrl = url,
                    productCode = fileAnalyser.getProductCode(),
                    upgradeBehavior = fileAnalyser.getUpgradeBehaviour(),
                    releaseDate = gitHubDetection?.releaseDate ?: releaseDate
                )
            } finally {
                file.delete()
                progress.clear()
            }
        }
        ParameterUrls.matchInstallers(
            downloadTasks,
            previousInstallers.map {
                it.copy(
                    installerType = remoteInstallerManifest.installerType?.toPerInstallerType() ?: it.installerType,
                    scope = remoteInstallerManifest.scope?.toPerScopeInstallerType() ?: it.scope
                )
            }
        ).forEach {
            installers += it.first.copy(
                installerUrl = it.second.installerUrl,
                installerSha256 = it.second.installerSha256.uppercase(),
                signatureSha256 = it.second.signatureSha256?.uppercase(),
                productCode = it.second.productCode,
                releaseDate = it.second.releaseDate
            )
        }
    }

    private suspend fun createFiles(): List<Pair<String, String>> = with(allManifestData) {
        return listOf(
            githubImpl.installerManifestName to InstallerManifestData.createInstallerManifest(),
            githubImpl.getDefaultLocaleManifestName() to DefaultLocaleManifestData.createDefaultLocaleManifest(),
            githubImpl.versionManifestName to VersionManifestData.createVersionManifest()
        ) + previousManifestData.remoteLocaleData.await()?.map { localeManifest ->
            val allLocale = additionalMetadataParam?.locales?.find { it.name.equals(other = "all", ignoreCase = true) }
            val metadataCurrentLocale = additionalMetadataParam?.locales?.find {
                it.name.equals(other = localeManifest.packageLocale, ignoreCase = true)
            }
            val schemas: Schemas by inject()
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = packageIdentifier,
                packageVersion = packageVersion,
                manifestVersion = schemas.manifestOverride ?: Schemas.manifestVersion,
                releaseNotes = allLocale?.releaseNotes ?: metadataCurrentLocale?.releaseNotes,
                releaseNotesUrl = allLocale?.releaseNotesUrl ?: metadataCurrentLocale?.releaseNotesUrl,
                documentations = allLocale?.documentations ?: metadataCurrentLocale?.documentations
            ).let {
                schemas.buildManifestString(
                    Schema.Locale,
                    EncodeConfig.yamlDefault.encodeToString(LocaleManifest.serializer(), it)
                )
            }
        }.orEmpty()
    }
}
