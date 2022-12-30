package data

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import org.kohsuke.github.GHRef
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GitHub
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleManifest
import schemas.InstallerManifest
import schemas.LocaleManifest
import schemas.ManifestBuilder
import schemas.Schemas
import schemas.SchemasImpl
import schemas.VersionManifest
import java.io.IOException

@Single
class GitHubImpl : KoinComponent {
    val github: GitHub = GitHubBuilder().withOAuthToken(token).build()
    private val sharedManifestData: SharedManifestData by inject()
    private val branchName: String
        get() = buildString {
            append(sharedManifestData.packageIdentifier)
            append("-")
            append(sharedManifestData.packageVersion)
            append("-")
            append(List(uniqueBranchIdentifierLength) { (('A'..'Z') + ('0'..'9')).random() }.joinToString(""))
        }

    fun getWingetPkgsFork(terminal: Terminal): GHRepository? {
        with(terminal) {
            return try {
                github.getRepository("${github.myself.login}/$wingetpkgs").also {
                    println(brightWhite("Found forked winget-pkgs repository: ${it.fullName}"))
                }
            } catch (_: IOException) {
                println(brightWhite("Fork of winget-pkgs not found. Forking..."))
                try {
                    github.getRepository("$Microsoft/$wingetpkgs").fork().also {
                        println(brightGreen("Forked winget-pkgs repository: ${it.fullName}"))
                    }
                } catch (_: IOException) {
                    println(red("Failed to fork winget-pkgs. Please try again or fork it manually."))
                    null
                }
            }
        }
    }

    fun getMicrosoftWingetPkgs(): GHRepository? {
        return try {
            github.getRepository("$Microsoft/$wingetpkgs")
        } catch (_: IOException) {
            null
        }
    }

    fun createBranch(repository: GHRepository?): GHRef? {
        return repository?.createRef(
            /* name = */ "refs/heads/$branchName",
            /* sha = */ repository.getBranch(repository.defaultBranch).shA1
        )
    }

    fun commitFiles(
        repository: GHRepository?,
        branch: GHRef?,
        installerManifest: InstallerManifest,
        defaultLocaleManifest: DefaultLocaleManifest,
        localeManifests: List<LocaleManifest>? = null,
        versionManifest: VersionManifest,
    ) {
        repository?.createCommit()
            ?.message("This is a test commit for Komac")
            ?.parent(branch?.getObject()?.sha)
            ?.tree(
                repository
                    .createTree()
                    .baseTree(repository.getBranch(branch?.ref).shA1)
                    .add(
                        ManifestBuilder.installerManifestGitHubPath,
                        buildManifestString(get<SchemasImpl>().installerSchema.id) {
                            appendLine(
                                YamlConfig.installer.encodeToString(
                                    InstallerManifest.serializer(),
                                    installerManifest
                                )
                            )
                        },
                        false
                    )
                    .add(
                        ManifestBuilder.defaultLocaleManifestGitHubPath,
                        buildManifestString(get<SchemasImpl>().defaultLocaleSchema.id) {
                            appendLine(
                                YamlConfig.other.encodeToString(
                                    DefaultLocaleManifest.serializer(),
                                    defaultLocaleManifest
                                )
                            )
                        },
                        false
                    )
                    .add(
                        ManifestBuilder.versionManifestGitHubPath,
                        buildManifestString(get<SchemasImpl>().versionSchema.id) {
                            appendLine(
                                YamlConfig.other.encodeToString(
                                    VersionManifest.serializer(),
                                    versionManifest
                                )
                            )
                        },
                        false
                    )
                    .apply {
                        localeManifests?.forEach {
                            add(
                                ManifestBuilder.getLocaleManifestGitHubPath(it.packageLocale),
                                buildManifestString(get<SchemasImpl>().localeSchema.id) {
                                    appendLine(YamlConfig.other.encodeToString(LocaleManifest.serializer(), it))
                                },
                                false
                            )
                        }
                    }
                    .create()
                    .sha
            )
            ?.create()
            ?.also { branch?.updateTo(it.shA1) }
    }

    private fun buildManifestString(schemaUrl: String, block: StringBuilder.() -> Unit): String {
        return buildString {
            appendLine(Schemas.Comments.createdBy)
            appendLine(Schemas.Comments.languageServer(schemaUrl))
            appendLine()
            block()
        }
    }

    companion object {
        private const val token = "TOKEN"
        const val Microsoft = "Microsoft"
        const val wingetpkgs = "winget-pkgs"
        private const val uniqueBranchIdentifierLength = 14
    }
}
