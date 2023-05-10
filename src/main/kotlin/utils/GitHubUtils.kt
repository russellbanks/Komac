package utils

import com.russellbanks.Komac.BuildConfig
import data.PreviousManifestData
import data.VersionUpdateState
import data.shared.Locale
import okio.ByteString.Companion.encodeUtf8
import org.kohsuke.github.GHRepository
import org.kohsuke.github.GHTreeEntry
import schemas.Schemas
import java.io.IOException
import java.time.Instant
import kotlin.random.Random

object GitHubUtils {
    /**
     * Returns the name of the YAML file containing the installer manifest for the given package identifier.
     *
     * @param identifier the package identifier to get the installer manifest name for
     * @return a string representing the name of the YAML file containing the installer manifest for the given
     * identifier
     */
    fun getInstallerManifestName(identifier: String) = "$identifier.installer.yaml"

    /**
     * Returns the name of the YAML file containing the localized manifest for the given package identifier and default
     * locale.
     *
     * @param identifier the package identifier to get the manifest name for
     * @param defaultLocale the default locale to get the manifest name for, if available
     * @param previousDefaultLocale the previously set default locale, if any
     * @return a string representing the name of the YAML file containing the localized manifest for the given
     * identifier and default locale
     */
    fun getDefaultLocaleManifestName(
        identifier: String,
        defaultLocale: String? = null,
        previousDefaultLocale: String? = PreviousManifestData.defaultLocaleManifest?.packageLocale
    ) = "$identifier.locale.${defaultLocale ?: previousDefaultLocale ?: Locale.defaultLocale}.yaml"

    /**
     * Returns the name of the YAML file containing the localized manifest for the given package identifier and locale.
     *
     * @param identifier the package identifier to get the manifest name for
     * @param locale the locale to get the manifest name for
     * @return a string representing the name of the YAML file containing the localized manifest for the given
     * identifier and locale
     */
    fun getLocaleManifestName(identifier: String, locale: String) = "$identifier.locale.$locale.yaml"

    /**
     * Returns the name of the YAML file containing the manifest for the given package identifier and version.
     *
     * @param identifier the package identifier to get the manifest name for
     * @return a string representing the name of the YAML file containing the manifest for the given identifier
     */
    fun getVersionManifestName(identifier: String) = "$identifier.yaml"

    /**
     * Returns the path to the directory containing the package manifest file for the given package identifier.
     *
     * @param identifier the package identifier to get the path for
     * @return a string representing the path to the directory containing the package manifest file for the given
     * identifier
     */
    fun getPackagePath(identifier: String): String {
        return "manifests/${identifier.first().lowercase()}/${identifier.replace('.', '/')}"
    }

    /**
     * Returns the path to the package versions directory for the given package identifier and version.
     *
     * @param identifier the package identifier to get the path for
     * @param version the package version to get the path for
     * @return a string representing the path to the package versions directory for the given identifier and version
     */
    fun getPackageVersionsPath(identifier: String, version: String) = "${getPackagePath(identifier)}/$version"

    /**
     * Returns a commit title based on the package identifier, package version, and version update state.
     *
     * @param identifier the package identifier being updated
     * @param version the new version of the package
     * @param updateState the state of the version update (e.g. "updated", "added", "removed")
     * @return a string representing the commit title, in the format "$updateState: $identifier version $version"
     */
    fun getCommitTitle(identifier: String, version: String, updateState: VersionUpdateState): String {
        return "$updateState: $identifier version $version"
    }

    /**
     * Returns a list of available package versions for the given package identifier, obtained from a GitHub repository.
     *
     * @param winGetPkgs the GitHub repository to search for package versions
     * @param identifier the package identifier to search for versions of
     * @return a list of available package versions for the given package identifier, or null if an IOException occurs
     * while accessing the GitHub repository.
     */
    fun getAllVersions(winGetPkgs: GHRepository, identifier: String): List<String>? = try {
        winGetPkgs
            .getTree(winGetPkgs.defaultBranch)
            .getEntry("manifests")
            .asTree()
            .getEntry(identifier.first().lowercase())
            .let { identifier.split('.').fold(it) { acc, s -> acc?.asTree()?.getEntry(s) } }
            ?.run {
                winGetPkgs.getTreeRecursive(sha, 1).tree
                    .filter { ghTreeEntry -> ghTreeEntry.path.count { it == '/' } in 1..2 }
                    .groupBy { it.path.substringBefore('/') }
                    .filterNot { entry -> entry.value.any { it.type == "tree" } }
                    .map(Map.Entry<String, List<GHTreeEntry>>::key)
                    .takeUnless(List<String>::isEmpty)
            }
    } catch (_: IOException) {
        null
    }

    /**
     * Returns a string with a message about a pull request created using the application's build version and a random
     * fruit emoji or a rocket emoji.
     *
     * @return a formatted string containing a message about a pull request created with the current build version
     * and a random emoji (fruit or rocket).
     */
    fun getPullRequestBody(): String {
        val fruits = listOf(
            "cherries", "grapes", "green_apple", "lemon", "melon", "pineapple", "strawberry", "tangerine", "watermelon"
        )
        return buildString {
            append("### Pull request has been created with ")
            append(
                System.getenv(Schemas.customToolEnv)
                    ?: "[${BuildConfig.appName}](${BuildConfig.projectUrl}) v${BuildConfig.appVersion}"
            )
            append(" ")
            append(if (Random.nextInt(30) == 0) ":${fruits.random()}:" else ":rocket:")
        }
    }

    /**
     * Generates a branch name for a package using the package identifier, package version, and current timestamp.
     * The branch name is generated by concatenating the package identifier, package version, and a hash of the
     * concatenation using the XXH3_64 algorithm. The hash is converted to uppercase before being appended to the
     * package identifier and version.
     *
     * @param packageIdentifier the identifier of the package
     * @param packageVersion the version of the package
     * @return a string representing the branch name for the package
     */
    fun getBranchName(packageIdentifier: String, packageVersion: String): String {
        val timestamp: Instant = Instant.now()
        val hash = "$packageIdentifier-$packageVersion-$timestamp".encodeUtf8().md5().hex().uppercase()
        return "$packageIdentifier-$packageVersion-$hash"
    }
}
