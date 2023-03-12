package detection

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import utils.getExtension

object ParameterUrls {
    fun assertUniqueUrlsCount(parameterUrls: List<Url>, previousUrls: List<Url>, terminal: Terminal) {
        val parameterUrlsSet = parameterUrls.toHashSet()
        val previousUrlsSet = previousUrls.toHashSet()
        if (parameterUrlsSet.size != previousUrlsSet.size) {
            throw CliktError(
                terminal.colors.danger(
                    buildString {
                        append("The number of unique installer urls ")
                        append(
                            when {
                                parameterUrlsSet.size > previousUrlsSet.size -> "is greater than"
                                parameterUrlsSet.size < previousUrlsSet.size -> "is less than"
                                else -> "does not match"
                            }
                        )
                        append(" the number of previous manifest urls")
                    }
                )
            )
        }
    }

    suspend fun assertUrlsValid(parameterUrls: List<Url>, terminal: Terminal, client: HttpClient) {
        parameterUrls.forEach { url ->
            data.shared.Url.isUrlValid(url, false, client)
                ?.let { throw CliktError(terminal.colors.danger("$it on $url")) }
        }
    }

    fun matchInstallers(
        newInstallers: List<InstallerManifest.Installer>,
        previousInstallers: List<InstallerManifest.Installer>
    ): Map<InstallerManifest.Installer, InstallerManifest.Installer> {
        return previousInstallers.associateWith { previousInstaller ->
            newInstallers.firstOrNull {
                it.architecture == previousInstaller.architecture &&
                    it.installerType == previousInstaller.installerType &&
                    (it.scope == previousInstaller.scope || it.scope == null)
            } ?: newInstallers.firstOrNull {
                it.architecture == previousInstaller.architecture &&
                    it.installerType == previousInstaller.installerType
            } ?: newInstallers.firstOrNull {
                it.installerType == previousInstaller.installerType
            } ?: newInstallers.firstOrNull {
                it.architecture == previousInstaller.architecture
            } ?: newInstallers.firstOrNull {
                it.installerUrl.getExtension() == previousInstaller.installerUrl.getExtension()
            } ?: previousInstaller // If no match was found, use the previous installer itself
        }
    }
}
