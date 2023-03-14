package utils

import com.russellbanks.Komac.BuildConfig
import data.VersionUpdateState
import data.shared.Locale
import data.shared.PackageVersion
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import schemas.Schemas
import java.io.IOException
import kotlin.random.Random

object GitHubUtils {
    fun getInstallerManifestName(identifier: String) = "$identifier.installer.yaml"

    fun getDefaultLocaleManifestName(
        identifier: String,
        defaultLocale: String? = null,
        previousDefaultLocale: String?
    ) = "$identifier.locale.${defaultLocale ?: previousDefaultLocale ?: Locale.defaultLocale}.yaml"

    fun getLocaleManifestName(identifier: String, locale: String) = "$identifier.locale.$locale.yaml"

    fun getVersionManifestName(identifier: String) = "$identifier.yaml"

    fun getPackagePath(identifier: String): String {
        return "manifests/${identifier.first().lowercase()}/${identifier.replace(".", "/")}"
    }

    fun getPackageVersionsPath(identifier: String, version: String) = "${getPackagePath(identifier)}/$version"

    fun getCommitTitle(identifier: String, version: String, updateState: VersionUpdateState): String {
        return "$updateState: $identifier version $version"
    }

    fun getAllVersions(winGetPkgs: GHRepository?, identifier: String) = try {
        winGetPkgs
            ?.getDirectoryContent(getPackagePath(identifier))
            ?.filter { it.name matches PackageVersion.regex }
            ?.filter(GHContent::isDirectory)
            ?.filterNot { it.name.all(Char::isLetter) }
            ?.takeIf(List<GHContent>::isNotEmpty)
            ?.map(GHContent::getName)
    } catch (_: IOException) {
        null
    }

    fun getPullRequestBody(): String {
        val fruits = listOf(
            "cherries", "grapes", "green_apple", "lemon", "melon", "pineapple", "strawberry", "tangerine", "watermelon"
        )
        return buildString {
            append("### Pull request has been created with ")
            append(System.getenv(Schemas.customToolEnv) ?: "${BuildConfig.appName} v${BuildConfig.appVersion}")
            append(" ")
            append(if (Random.nextInt(30) == 0) ":${fruits.random()}:" else ":rocket:")
        }
    }
}
