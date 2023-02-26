package network

import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import io.ktor.http.fullPath
import schemas.manifest.InstallerManifest

fun Url.findArchitecture(): InstallerManifest.Installer.Architecture? {
    val archInUrl = Regex("(x86_64|i?[3-6]86|x\\d+|arm(?:64)?|aarch(?:64)?|amd64?)", RegexOption.IGNORE_CASE)
        .find(fullPath)?.groupValues?.last()
    return when (archInUrl?.lowercase()) {
        "aarch" -> InstallerManifest.Installer.Architecture.ARM
        "aarch64" -> InstallerManifest.Installer.Architecture.ARM64
        "x86_64", "amd64" -> InstallerManifest.Installer.Architecture.X64
        "i386", "386", "i486", "486", "i586", "586", "i686", "686" -> InstallerManifest.Installer.Architecture.X86
        else -> {
            try {
                InstallerManifest.Installer.Architecture.valueOf(archInUrl?.uppercase() ?: "")
            } catch (_: IllegalArgumentException) {
                null
            }
        }
    }
}

fun Url.getExtension(): String {
    return fullPath.substringAfterLast(".").split(Regex("[^A-Za-z0-9]")).firstOrNull() ?: "winget-tmp"
}

fun Url.getFileName(): String? = pathSegments.findLast { it.endsWith(getExtension()) }

fun Url.getFileNameWithoutExtension(): String? = getFileName()?.removeSuffix(getExtension())

fun Url.findScope(): InstallerManifest.Installer.Scope? {
    return when {
        fullPath.contains(other = "user", ignoreCase = true) -> InstallerManifest.Installer.Scope.User
        fullPath.contains(other = "machine", ignoreCase = true) -> InstallerManifest.Installer.Scope.Machine
        else -> null
    }
}

fun HttpStatusCode.isRedirect(): Boolean {
    return value in HttpStatusCode.MovedPermanently.value..HttpStatusCode.PermanentRedirect.value
}
