package utils

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.rendering.Theme
import data.shared.InstallerUrl
import io.ktor.http.Url
import java.lang.System
import schemas.manifest.InstallerManifest

object UrlsToInstallerMatcher {
    suspend fun assertUrlsValid(parameterUrls: Set<Url>, theme: Theme) {
        val errorList = parameterUrls.mapNotNull { url ->
            InstallerUrl.getError(url.toString())?.let { error -> url to error }
        }
        if (errorList.isNotEmpty()) {
            throw CliktError(
                theme.danger(errorList.joinToString(System.lineSeparator()) { (url, error) -> "$error on $url" }),
                statusCode = 1
            )
        }
    }

    fun matchInstallers(
        newInstallers: List<InstallerManifest.Installer>,
        previousInstallers: List<InstallerManifest.Installer>
    ): Map<InstallerManifest.Installer, InstallerManifest.Installer> {
        val foundArchitectures = newInstallers.mapNotNull { installer ->
            installer.installerUrl.findArchitecture()?.let { architecture ->
                installer.installerUrl to architecture
            }
        }.toMap()
        val foundScopes = newInstallers.mapNotNull { installer ->
            installer.installerUrl.findScope()?.let { scope ->
                installer.installerUrl to scope
            }
        }.toMap()
        return previousInstallers.associateWith { previousInstaller ->
            var maxScore = 0
            lateinit var bestMatch: InstallerManifest.Installer
            for (newInstaller in newInstallers) {
                var score = 0
                if (newInstaller.architecture == previousInstaller.architecture) score++
                if (foundArchitectures[newInstaller.installerUrl] == previousInstaller.architecture) score++
                if (newInstaller.installerType == previousInstaller.installerType) score++
                if (newInstaller.scope == previousInstaller.scope) score++

                val isNewArchitecture = foundArchitectures[newInstaller.installerUrl] !in foundArchitectures.values
                val isNewScope = foundScopes.isNotEmpty() && foundScopes[newInstaller.installerUrl] !in foundScopes.values

                if (score > maxScore || (score == maxScore && (isNewArchitecture || isNewScope))) {
                    maxScore = score
                    bestMatch = newInstaller
                }
            }
            bestMatch
        }
    }
}
