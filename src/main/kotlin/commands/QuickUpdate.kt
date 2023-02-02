package commands
import Errors
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.installer.InstallerType
import data.shared.Locale
import data.shared.PackageIdentifier.packageIdentifierPrompt
import data.shared.PackageVersion.packageVersionPrompt
import data.shared.Url.installerDownloadPrompt
import detection.GitHubDetection
import detection.ParameterUrls
import input.FileWriter.writeFiles
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import network.Http
import network.HttpUtils
import network.HttpUtils.decodeHex
import network.HttpUtils.detectArchitectureFromUrl
import network.HttpUtils.detectScopeFromUrl
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.Schema
import schemas.Schemas
import schemas.SchemasImpl
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.YamlConfig
import token.TokenStore
import utils.FileUtils
import utils.Hashing.hash
import java.io.File
import java.io.IOException
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

class QuickUpdate : CliktCommand(name = "update"), KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val versionManifestData: VersionManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private lateinit var previousManifestData: PreviousManifestData
    private lateinit var files: List<Pair<String, String?>>
    private val githubImpl: GitHubImpl by inject()
    private val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    private val packageIdentifier: String? by option("--id", "--package-identifier")
    private val packageVersion: String? by option("--version", "--package-version")
    private val urls: List<Url>? by option().convert { Url(it.removeSuffix("/")).decodeHex() }.split(",")
    private val manifestVersion: String? by option()
    private val submit: Boolean by option().flag(default = false)
    private val tokenParameter: String? by option("-t", "--token")

    override fun run(): Unit = runBlocking {
        manifestVersion?.let { get<SchemasImpl>().manifestOverride = it }
        with(currentContext.terminal) {
            if (isCIEnvironment) {
                info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
                tokenParameter?.let {
                    get<TokenStore>().useTokenParameter(it).let { isTokenValid ->
                        if (!isTokenValid) {
                            throw CliktError(message = colors.danger("${Errors.error} Invalid token"), statusCode = 1)
                        }
                    }
                }
            }
            packageIdentifierPrompt(packageIdentifier, isCIEnvironment)
            previousManifestData = get()
            if (sharedManifestData.updateState == VersionUpdateState.NewPackage) {
                warning("${sharedManifestData.packageIdentifier} is not in the ${GitHubImpl.wingetpkgs} repository.")
                warning("Please use the 'new' command to create a new manifest.")
                return@runBlocking
            }
            launch {
                packageVersionPrompt(packageVersion, isCIEnvironment)
                previousManifestData.remoteInstallerDataJob.join()
                loopThroughInstallers(parameterUrls = urls, isCIEnvironment = isCIEnvironment)
                createFiles()
                if (submit) {
                    commit()
                    pullRequest()
                } else if (!isCIEnvironment) {
                    pullRequestPrompt(sharedManifestData).also { manifestResultOption ->
                        when (manifestResultOption) {
                            ManifestResultOption.PullRequest -> {
                                commit()
                                pullRequest()
                            }
                            ManifestResultOption.WriteToFiles -> writeFiles(files)
                            else -> echo("Exiting")
                        }
                    }
                }
            }
        }
    }

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: List<Url>? = null,
        isCIEnvironment: Boolean = false
    ) = coroutineScope {
        val remoteInstallerManifest = previousManifestData.remoteInstallerData
        val previousInstallers = remoteInstallerManifest!!.installers
        val previousUrls = previousInstallers.map { it.installerUrl }
        if (parameterUrls != null) {
            ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls, this@loopThroughInstallers)
            ParameterUrls.assertUrlsValid(parameterUrls, this@loopThroughInstallers)
            val downloadTasks = mutableListOf<InstallerManifest.Installer>()
            val client = get<Http>().client
            parameterUrls.distinct().forEach { url ->
                val file = withContext(Dispatchers.IO) {
                    val formattedDate = DateTimeFormatter.ofPattern("yyyy.MM.dd-hh.mm.ss").format(LocalDateTime.now())
                    File.createTempFile(
                        "${sharedManifestData.packageIdentifier} v${sharedManifestData.packageVersion} - $formattedDate",
                        ".${HttpUtils.getURLExtension(url)}"
                    )
                }
                val progress = currentContext.terminal.progressAnimation {
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
                val fileUtils = FileUtils(file)
                downloadTasks += try {
                    InstallerManifest.Installer(
                        architecture = detectArchitectureFromUrl(url) ?: fileUtils.getArchitecture(),
                        installerType = fileUtils.getInstallerType(),
                        scope = detectScopeFromUrl(url),
                        installerSha256 = file.hash(),
                        signatureSha256 = fileUtils.getSignatureSha256(),
                        installerUrl = url,
                        productCode = fileUtils.getProductCode(),
                        upgradeBehavior = fileUtils.getUpgradeBehaviour(),
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

    private suspend fun createFiles() {
        sharedManifestData.defaultLocale = previousManifestData.remoteVersionData!!.defaultLocale
        files = listOf(
            githubImpl.installerManifestName to installerManifestData.createInstallerManifest(),
            githubImpl.defaultLocaleManifestName to defaultLocaleManifestData.createDefaultLocaleManifest(),
            githubImpl.versionManifestName to versionManifestData.createVersionManifest()
        ) + previousManifestData.remoteLocaleData?.map { localeManifest ->
            githubImpl.getLocaleManifestName(localeManifest.packageLocale) to localeManifest.copy(
                packageIdentifier = sharedManifestData.packageIdentifier,
                packageVersion = sharedManifestData.packageVersion,
                manifestVersion = get<SchemasImpl>().localeSchema.properties.manifestVersion.default
            ).let {
                Schemas.buildManifestString(
                    Schema.Locale,
                    YamlConfig.default.encodeToString(LocaleManifest.serializer(), it)
                )
            }
        }.orEmpty()
    }

    private suspend fun Terminal.commit() {
        previousManifestData.remoteVersionDataJob.join()
        previousManifestData.remoteLocaleDataJob.join()
        previousManifestData.remoteDefaultLocaleDataJob.join()
        val repository = githubImpl.getWingetPkgsFork(terminal = this) ?: return
        val ref = githubImpl.createBranchFromDefaultBranch(repository = repository, terminal = this) ?: return
        githubImpl.commitFiles(
            repository = repository,
            branch = ref,
            files = files.map { "${githubImpl.baseGitHubPath}/${it.first}" to it.second }
        )
    }

    private suspend fun Terminal.pullRequest() {
        val ghRepository = githubImpl.getMicrosoftWingetPkgs() ?: return
        try {
            ghRepository.createPullRequest(
                /* title = */ githubImpl.getCommitTitle(),
                /* head = */ "${githubImpl.github.await().myself.login}:${githubImpl.pullRequestBranch?.ref}",
                /* base = */ ghRepository.defaultBranch,
                /* body = */ githubImpl.getPullRequestBody()
            ).also { success("Pull request created: ${it.htmlUrl}") }
        } catch (ioException: IOException) {
            danger(ioException.message ?: "Failed to create pull request")
        }
    }
}
