package utils

import data.shared.Locale
import data.shared.PackageVersion
import org.kohsuke.github.GHContent
import org.kohsuke.github.GHRepository
import java.io.IOException

object GitHubUtils {
    fun getInstallerManifestName(identifier: String) = "$identifier.installer.yaml"

    fun getDefaultLocaleManifestName(
        identifier: String,
        defaultLocale: String? = null,
        previousDefaultLocale: String?
    ): String {
        return "$identifier.locale.${defaultLocale ?: previousDefaultLocale ?: Locale.defaultLocale}.yaml"
    }

    fun getLocaleManifestName(identifier: String, locale: String): String {
        return "$identifier.locale.$locale.yaml"
    }

    fun getVersionManifestName(identifier: String) = "$identifier.yaml"

    fun getPackagePath(identifier: String): String {
        return "manifests/${identifier.first().lowercase()}/${identifier.replace(".", "/")}"
    }

    fun getPackageVersionsPath(identifier: String, version: String): String {
        return "${getPackagePath(identifier)}/$version"
    }

    fun getAllVersions(winGetPkgs: GHRepository?, identifier: String): List<String>? {
        return try {
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
    }
}
