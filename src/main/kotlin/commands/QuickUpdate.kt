package commands

import Errors
import Errors.doesNotExistError
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.clikt.parameters.options.convert
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.options.split
import com.github.ajalt.clikt.parameters.options.validate
import com.github.ajalt.mordant.animation.ProgressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import data.AllManifestData
import data.DefaultLocaleManifestData
import data.GitHubImpl
import data.InstallerManifestData
import data.PreviousManifestData
import data.VersionManifestData
import data.VersionUpdateState
import data.installer.InstallerType
import data.shared.Locale
import data.shared.PackageIdentifier
import data.shared.PackageVersion
import data.shared.PackageVersion.getHighestVersion
import data.shared.Url.installerDownloadPrompt
import data.shared.getUpdateState
import detection.ParameterUrls
import detection.github.GitHubDetection
import input.FileWriter
import input.ManifestResultOption
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
import org.kohsuke.github.GitHub
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
import utils.GitHubUtils
import utils.Hashing.hash
import utils.ManifestUtils.formattedManifestLinesSequence
import utils.ManifestUtils.updateVersionInString
import utils.findArchitecture
import utils.findScope
import java.io.File
import java.time.LocalDate
import java.time.ZoneOffset

class QuickUpdate : CliktCommand(name = "update") {
    private val allManifestData = AllManifestData()
    private val tokenStore = TokenStore()
    private lateinit var previousManifestData: PreviousManifestData
    private val client = Http.client
    private val gitHubImpl by lazy { GitHubImpl(tokenStore.token as String, client) }
    private val isCIEnvironment = System.getenv("CI")?.toBooleanStrictOrNull() == true
    private val packageIdentifierParam: String? by option("--id", "--package-identifier")
    private val packageVersionParam: String? by option("--version", "--package-version")
    private val urls: List<Url>? by option().convert { Url(it) }.split(",")
    private val manifestOverride: String? by option().validate {
        require("^\\d+\\.\\d+\\.\\d+$".toRegex() matches it) { "Manifest version must be in the format X.X.X" }
    }
    private val submit: Boolean by option().flag(default = false)
    private val tokenParameter: String? by option("-t", "--token", envvar = "GITHUB_TOKEN").validate {
        require(GitHub.connectUsingOAuth(it).isCredentialValid) {
            colors.danger("The token is invalid or has expired")
        }
    }
    private val additionalMetadataParam by option(hidden = true).convert {
        EncodeConfig.jsonDefault.decodeFromString(AdditionalMetadata.serializer(), it)
    }

    override fun run(): Unit = runBlocking {
        tokenParameter?.let { tokenStore.useTokenParameter(it) }
        with(allManifestData) {
            if (tokenStore.token == null) prompt(Token).also { tokenStore.putToken(it) }
            if (isCIEnvironment) {
                info("CI environment detected! Komac will throw errors instead of prompting on invalid input")
            }
            packageIdentifier = prompt(PackageIdentifier, parameter = packageIdentifierParam)
            if (!tokenStore.isTokenValid.await()) tokenStore.invalidTokenPrompt(currentContext.terminal)
            allVersions = GitHubUtils.getAllVersions(gitHubImpl.getMicrosoftWinGetPkgs(), packageIdentifier)
            latestVersion = allVersions?.getHighestVersion()?.also {
                if (!isCIEnvironment) {
                    info("Found $packageIdentifier in the winget-pkgs repository")
                    info("Found latest version: $it")
                }
            }
            previousManifestData = PreviousManifestData(packageIdentifier, latestVersion, gitHubImpl.getMicrosoftWinGetPkgs())
            if (updateState == VersionUpdateState.NewPackage) {
                throw doesNotExistError(packageIdentifier, packageVersion, true)
            }
            packageVersion = prompt(PackageVersion, parameter = packageVersionParam)
            gitHubImpl.promptIfPullRequestExists(
                identifier = packageIdentifier,
                version = packageVersion,
                terminal = currentContext.terminal
            )
            updateState = getUpdateState(updateState, packageIdentifier, packageVersion, latestVersion, gitHubImpl)
            currentContext.terminal.loopThroughInstallers(parameterUrls = urls, isCIEnvironment = isCIEnvironment)
            val files = createFiles(packageIdentifier, packageVersion, defaultLocale)
            files.values.forEach { manifest ->
                formattedManifestLinesSequence(manifest, currentContext.terminal.colors).forEach(::echo)
            }
            if (submit) {
                gitHubImpl.commitAndPullRequest(
                    files = files,
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    updateState = updateState,
                    terminal = currentContext.terminal
                )
            } else if (!isCIEnvironment) {
                currentContext.terminal.pullRequestPrompt(packageIdentifier, packageVersion).also { manifestResultOption ->
                    when (manifestResultOption) {
                        ManifestResultOption.PullRequest -> {
                            gitHubImpl.commitAndPullRequest(
                                files = files,
                                packageIdentifier = packageIdentifier,
                                packageVersion = packageVersion,
                                updateState = updateState,
                                terminal = currentContext.terminal
                            ).let {
                                if (it != null) {
                                    success("Pull request created: ${it.htmlUrl}")
                                } else {
                                    throw CliktError("Failed to create pull request", statusCode = 1)
                                }
                            }
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

    private suspend fun Terminal.loopThroughInstallers(
        parameterUrls: List<Url>? = null,
        isCIEnvironment: Boolean = false
    ) = with(allManifestData) {
        val remoteInstallerManifest = previousManifestData.remoteInstallerData.await()
            ?: throw CliktError(colors.danger("Failed to retrieve previous installers"), statusCode = 1)
        if (parameterUrls != null) {
            loopParameterUrls(parameterUrls = parameterUrls, previousInstallerManifest = remoteInstallerManifest)
        } else if (isCIEnvironment) {
            throw CliktError(colors.danger("${Errors.error} No installers have been provided"), statusCode = 1)
        } else {
            remoteInstallerManifest.installers.forEachIndexed { index, installer ->
                info("Installer Entry ${index.inc()}/${remoteInstallerManifest.installers.size}")
                listOf(
                    InstallerManifest.Installer.Architecture::class.simpleName to installer.architecture,
                    InstallerType.const to (installer.installerType ?: remoteInstallerManifest.installerType),
                    InstallerManifest.Scope::class.simpleName to (installer.scope ?: remoteInstallerManifest.scope),
                    Locale.installerLocaleConst to
                        (installer.installerLocale ?: remoteInstallerManifest.installerLocale)
                ).forEach { (promptType, value) ->
                    value?.let {
                        echo("  $promptType: ${colors.info(it.toString())}")
                    }
                }
                echo()
                installerDownloadPrompt(allManifestData, client, gitHubImpl)
                if (!skipAddInstaller) {
                    InstallerManifestData.addInstaller(allManifestData, previousManifestData)
                } else {
                    skipAddInstaller = false
                }
            }
        }
    }

    private suspend fun Terminal.loopParameterUrls(
        parameterUrls: List<Url>,
        previousInstallerManifest: InstallerManifest
    ) = with(allManifestData) {
        val previousInstallers = previousInstallerManifest.installers
        val previousUrls = previousInstallers.map { it.installerUrl }
        ParameterUrls.assertUniqueUrlsCount(parameterUrls, previousUrls, this@loopParameterUrls)
        ParameterUrls.assertUrlsValid(parameterUrls, this@loopParameterUrls, client)
        val downloadTasks = mutableListOf<InstallerManifest.Installer>()
        parameterUrls.distinct().forEach { url ->
            val file = FileUtils.createTempFile(identifier = packageIdentifier, version = packageVersion, url = url)
            val progress = HttpUtils.getDownloadProgressBar(url, this@loopParameterUrls).also(ProgressAnimation::start)
            gitHubDetection = parameterUrls
                .firstOrNull { it.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true) }
                ?.let { GitHubDetection(it, gitHubImpl, client) }
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
                    installerType = previousInstallerManifest.installerType?.toPerInstallerType() ?: it.installerType,
                    scope = previousInstallerManifest.scope?.toPerScopeInstallerType() ?: it.scope
                )
            }
        ).forEach { (previousInstaller, newInstaller) ->
            installers += previousInstaller.copy(
                installerUrl = newInstaller.installerUrl,
                installerSha256 = newInstaller.installerSha256.uppercase(),
                signatureSha256 = newInstaller.signatureSha256?.uppercase(),
                productCode = newInstaller.productCode,
                releaseDate = newInstaller.releaseDate,
                nestedInstallerFiles = previousInstaller.nestedInstallerFiles?.map {
                    it.copy(relativeFilePath = it.relativeFilePath.updateVersionInString(allVersions, packageVersion))
                }
            )
        }
    }

    private suspend fun createFiles(
        packageIdentifier: String,
        packageVersion: String,
        defaultLocale: String?
    ): Map<String, String> {
        return mapOf(
            GitHubUtils.getInstallerManifestName(packageIdentifier) to InstallerManifestData.createInstallerManifest(allManifestData, previousManifestData.remoteInstallerData.await(), manifestOverride),
            GitHubUtils.getDefaultLocaleManifestName(packageIdentifier, defaultLocale, previousManifestData.remoteDefaultLocaleData.await()?.packageLocale) to DefaultLocaleManifestData.createDefaultLocaleManifest(allManifestData, previousManifestData, manifestOverride),
            GitHubUtils.getVersionManifestName(packageIdentifier) to VersionManifestData.createVersionManifest(
                allManifestData = allManifestData,
                manifestOverride = manifestOverride,
                previousVersionData = previousManifestData.previousVersionData
            )
        ) + previousManifestData.remoteLocaleData.await()?.mapNotNull { localeManifest ->
            additionalMetadataParam?.locales
                ?.find { it.name.equals(other = localeManifest.packageLocale, ignoreCase = true) }
                ?.let { metadataCurrentLocale ->
                    val allLocale = additionalMetadataParam?.locales?.find {
                        it.name.equals(other = "all", ignoreCase = true)
                    }
                    GitHubUtils.getLocaleManifestName(packageIdentifier, localeManifest.packageLocale) to run {
                        val copyLocaleManifest = localeManifest.copy(
                            packageIdentifier = packageIdentifier,
                            packageVersion = packageVersion,
                            manifestVersion = manifestOverride ?: Schemas.manifestVersion,
                            releaseNotes = allLocale?.releaseNotes ?: metadataCurrentLocale.releaseNotes,
                            releaseNotesUrl = allLocale?.releaseNotesUrl ?: metadataCurrentLocale.releaseNotesUrl,
                            documentations = allLocale?.documentations ?: metadataCurrentLocale.documentations
                        )
                        Schemas.buildManifestString(
                            schema = Schema.Locale,
                            rawString = EncodeConfig.yamlDefault.encodeToString(
                                serializer = LocaleManifest.serializer(),
                                value = copyLocaleManifest
                            )
                        )
                    }
                }
        }.orEmpty()
    }
}
