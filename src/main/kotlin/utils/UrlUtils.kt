package utils

import io.ktor.client.HttpClient
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import io.ktor.http.fullPath
import schemas.manifest.InstallerManifest

/**
 * Tries to extract the architecture from the URL string and returns it as an
 * [InstallerManifest.Installer.Architecture] enum value.
 *
 * @return an [InstallerManifest.Installer.Architecture] enum value if an architecture can be found, otherwise null
 */
fun Url.findArchitecture(): InstallerManifest.Installer.Architecture? {
    val archInUrl = Regex("(x86_64|i?[3-6]86|x\\d+|arm(?:64)?|aarch(?:64)?|amd64?)", RegexOption.IGNORE_CASE)
        .find(fullPath)
        ?.run { groupValues.last() }
    return when (archInUrl?.lowercase()) {
        "aarch" -> InstallerManifest.Installer.Architecture.ARM
        "aarch64" -> InstallerManifest.Installer.Architecture.ARM64
        "x86_64", "amd64" -> InstallerManifest.Installer.Architecture.X64
        "i386", "386", "i486", "486", "i586", "586", "i686", "686" -> InstallerManifest.Installer.Architecture.X86
        else -> {
            try {
                InstallerManifest.Installer.Architecture.valueOf(archInUrl?.uppercase().orEmpty())
            } catch (_: IllegalArgumentException) {
                null
            }
        }
    }
}

/**
 * Returns the extension of this Url.
 *
 * The extension is defined as the substring after the last occurrence of the '.' character in the full path of the Url.
 * If there are any non-alphanumeric characters in the extension, they are removed.
 * If no extension is found, the default value "winget-tmp" is returned.
 *
 * @return The extension of this Url, or "winget-tmp" if no extension is found.
 */
fun Url.getExtension() = fullPath
    .substringAfterLast(".")
    .split("[^A-Za-z0-9]".toRegex())
    .firstOrNull()
    ?: "winget-tmp"

/**
 * Returns the filename of this URL, including the extension, if it has one.
 *
 * The filename is determined by searching the URL's path segments for the last segment that ends with the same
 * extension as the URL's extension, obtained by calling the getExtension() function. If no such segment is found, null
 * is returned.
 *
 * @return the filename of this URL, including the extension, or null if no such filename can be determined
 */
fun Url.getFileName(): String? = pathSegments.findLast { it.endsWith(".${getExtension()}") }

/**
 * Returns the filename of this URL without the extension, if it has one.
 *
 * The filename without the extension is determined by calling the getFileName() function to obtain the full filename,
 * then removing the extension by calling the removeSuffix() function with the URL's extension, obtained by calling
 * the getExtension() function. If the getFileName() function returns null, null is returned.
 *
 * @return the filename of this URL without the extension, or null if no such filename can be determined
 */
fun Url.getFileNameWithoutExtension(): String? = getFileName()?.removeSuffix(".${getExtension()}")

/**
 * Determines the installation scope of the installer manifest at this URL, if any.
 *
 * The installation scope is determined by searching the URL's full path for the strings "user" or "machine",
 * ignoring case. If "user" is found, the installation scope is [InstallerManifest.Installer.Scope.User].
 * If "machine" is found, the installation scope is [InstallerManifest.Installer.Scope.Machine].
 * If neither string is found, null is returned.
 *
 * @return the installation scope of the installer manifest at this URL, or null if no scope can be determined
 */
fun Url.findScope(): InstallerManifest.Installer.Scope? {
    return when {
        fullPath.contains(other = "user", ignoreCase = true) -> InstallerManifest.Installer.Scope.User
        fullPath.contains(other = "machine", ignoreCase = true) -> InstallerManifest.Installer.Scope.Machine
        else -> null
    }
}

/**
 * Retrieves the final URL after all redirects have been followed, given a [client] of [HttpClient].
 *
 * @param client a [HttpClient] instance to make the request.
 * @return the final URL after all redirects have been followed.
 */
suspend fun Url.getRedirectedUrl(client: HttpClient): Url {
    client.config { followRedirects = false }.use { noRedirectClient ->
        var redirectedInstallerUrl: Url = this
        var response: HttpResponse? = noRedirectClient.head(this)

        var status: HttpStatusCode? = response?.status
        var location: String? = response?.run { headers[HttpHeaders.Location] }
        var redirectCount = 0
        while (status?.isRedirect() == true && location != null && redirectCount < 5) {
            redirectedInstallerUrl = Url(location)
            response = noRedirectClient.head(redirectedInstallerUrl)
            status = response.status
            location = response.headers[HttpHeaders.Location]
            redirectCount++
        }
        return redirectedInstallerUrl
    }
}

/**
 * Determines whether this HTTP status code represents a redirect.
 *
 * A status code is considered a redirect if its value is between HttpStatusCode.MultipleChoices.value (300)
 * and HttpStatusCode.PermanentRedirect.value (308), inclusive.
 *
 * @return true if this status code represents a redirect, false otherwise
 */
fun HttpStatusCode.isRedirect(): Boolean {
    return value in HttpStatusCode.MultipleChoices.value..HttpStatusCode.PermanentRedirect.value
}
