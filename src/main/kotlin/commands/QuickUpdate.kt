package commands

import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.clikt.parameters.options.validate
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandUtils.prompt
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.ManifestUtils.formattedManifestLinesFlow
import data.PreviousManifestData
import data.SharedManifestData
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
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.runBlocking
import network.Http
import network.HttpUtils
import network.HttpUtils.detectScopeFromUrl
import network.HttpUtils.getArchitectureFromUrl
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
import token.TokenStore
import utils.FileAnalyser
import utils.FileUtils
import utils.Hashing.hash
import java.io.File
import java.time.LocalDate
import java.time.ZoneId

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val tokenStore: TokenStore by inject()
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var previousManifestData: PreviousManifestData
    private val githubImpl: GitHubImpl by inject()
    private val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    private val packageIdentifier: String? by option("--id", "--package-identifier")
    private val packageVersion: String? by option("--version", "--package-version")
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
    private val additionalMetadata by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }

    override fun run(): Unit = runBlocking {
        manifestVersion?.let { get<Schemas>().manifestOverride = it }
        with(currentContext.terminal) {
            if (tokenStore.token == null) {
                tokenStore.promptForToken(this)
            }
            if (isCIEnvironment) {
                info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
                tokenParameter?.let { get<TokenStore>().useTokenParameter(it) }
            }
            sharedManifestData.packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifier)
            if (!tokenStore.isTokenValid.await()) {
                tokenStore.invalidTokenPrompt(this)
                echo()
            }
            sharedManifestData.latestVersion = getLatestVersion(sharedManifestData.packageIdentifier, !isCIEnvironment)
            previousManifestData = get()
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
                throw CliktError(
                    colors.warning(
                        buildString {
                            appendLine("${sharedManifestData.packageIdentifier} does not exist in the ${GitHubImpl.wingetpkgs} repository.")
                            appendLine("Please use the 'new' command to create a new manifest.")
                        }
                    ),
                    statusCode = 1
                )
            }
            sharedManifestData.packageVersion = prompt(PackageVersion, parameter = packageVersion)
            githubImpl.promptIfPullRequestExists(
                identifier = sharedManifestData.packageIdentifier,
                version = sharedManifestData.packageVersion,
                terminal = this
            )
            PackageVersion.setUpgradeState(PackageVersion.sharedManifestData)
            loopThroughInstallers(parameterUrls = urls, isCIEnvironment = isCIEnvironment)
            val files = createFiles()
            files.forEach { manifest ->
                formattedManifestLinesFlow(manifest.second, colors).collect { echo(it) }
            }
            if (submit) {
                githubImpl.commitAndPullRequest(files, this@with)
            } else if (!isCIEnvironment) {
                pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> githubImpl.commitAndPullRequest(files, this@with)
                        ManifestResultOption.WriteToFiles -> FileWriter.writeFiles(files, this@with)
                        else -> return@also
                    }
                }
            } else {
                FileWriter.writeFilesToDirectory(File(System.getProperty("user.dir")), files, this@with)
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: List<Url>? = null,
        isCIEnvironment: Boolean = false
    ) = coroutineScope {
        val remoteInstallerManifest = previousManifestData.remoteInstallerData.await()!!
        val previousInstallers = remoteInstallerManifest.installers
        val previousUrls = previousInstallers.map { it.installerUrl }
        if (parameterUrls != null) {
            ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls, this@loopThroughInstallers)
            ParameterUrls.assertUrlsValid(parameterUrls, this@loopThroughInstallers)
            val downloadTasks = mutableListOf<InstallerManifest.Installer>()
            val client = get<Http>().client
            parameterUrls.distinct().forEach { url ->
                val file = FileUtils.createTempFile(
                    identifier = sharedManifestData.packageIdentifier,
                    version = sharedManifestData.packageVersion,
                    url = url
                )
                val progress = progressAnimation {
                    HttpUtils.getFileName(url)?.let { text(it) }
                    percentage()
                    progressBar()
                    completed()
                    speed("B/s")
                    timeRemaining()
                }
                progress.start()
                sharedManifestData.gitHubDetection = parameterUrls.firstOrNull {
                    it.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true)
                }?.let { GitHubDetection(it) }
                var releaseDate: LocalDate? = null
                client.prepareGet(url).execute { httpResponse ->
                    releaseDate = httpResponse.lastModified()?.let {
                        LocalDate.ofInstant(it.toInstant(), ZoneId.systemDefault())
                    }
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
                        architecture = getArchitectureFromUrl(url) ?: fileAnalyser.getArchitecture(),
                        installerType = fileAnalyser.getInstallerType(),
                        scope = detectScopeFromUrl(url),
                        installerSha256 = file.hash(),
                        signatureSha256 = fileAnalyser.getSignatureSha256(),
                        installerUrl = url,
                        productCode = fileAnalyser.getProductCode(),
                        upgradeBehavior = fileAnalyser.getUpgradeBehaviour(),
                        releaseDate = sharedManifestData.gitHubDetection?.releaseDate?.await() ?: releaseDate
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
                installerManifestData.installers += it.first.copy(
                    installerUrl = it.second.installerUrl,
                    installerSha256 = it.second.installerSha256.uppercase(),
                    signatureSha256 = it.second.signatureSha256?.uppercase(),
                    productCode = it.second.productCode,
                    releaseDate = it.second.releaseDate
                )
            }
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
                installerManifestData.addInstaller()
            }
        }
    }

    private suspend fun createFiles(): List<Pair<String, String>> {
        return listOf(
            githubImpl.installerManifestName to installerManifestData.createInstallerManifest(),
            githubImpl.getDefaultLocaleManifestName() to defaultLocaleManifestData.createDefaultLocaleManifest(),
            githubImpl.versionManifestName to VersionManifestData.createVersionManifest()
        ) + previousManifestData.remoteLocaleData.await()?.map { localeManifest ->
            val allLocale = additionalMetadata?.locales?.find { it.name.equals(other = "all", ignoreCase = true) }
            val metadataCurrentLocale = additionalMetadata?.locales?.find {
                it.name.equals(other = localeManifest.packageLocale, ignoreCase = true)
            }
            val schemas: Schemas by inject()
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
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
