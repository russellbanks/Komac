package detection

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.TerminalColors
import io.ktor.client.HttpClient
import io.ktor.http.Url
import schemas.manifest.InstallerManifest
import utils.getExtension

object ParameterUrls {
    fun assertUniqueUrlsCount(parameterUrls: Set<Url>, previousUrls: Set<Url>, colors: TerminalColors) {
        if (parameterUrls.size != previousUrls.size) {
            throw CliktError(
                colors.danger(
                    buildString {
                        append("The number of unique installer urls ")
                        append(
                            when {
                                parameterUrls.size > previousUrls.size -> "is greater than"
                                parameterUrls.size < previousUrls.size -> "is less than"
                                else -> "does not match"
                            }
                        )
                        append(" the number of previous manifest urls")
                    }
                ),
                statusCode = 1
            )
        }
    }

    suspend fun assertUrlsValid(parameterUrls: Set<Url>, client: HttpClient, colors: TerminalColors) {
        val errorList = parameterUrls.mapNotNull { url ->
            data.shared.Url.isUrlValid(url, false, client)?.let { error -> url to error }
        }
        if (errorList.isNotEmpty()) {
            throw CliktError(
                colors.danger(errorList.joinToString(System.lineSeparator()) { (url, error) -> "$error on $url" }),
                statusCode = 1
            )
        }
    }

    fun matchInstallers(
        newInstallers: List<InstallerManifest.Installer>,
        previousInstallers: List<InstallerManifest.Installer>
    ): Map<InstallerManifest.Installer, InstallerManifest.Installer> {
        val result = hashMapOf<InstallerManifest.Installer, InstallerManifest.Installer>()
        for (previousInstaller in previousInstallers) {
            var newInstaller: InstallerManifest.Installer? = newInstallers.firstOrNull {
                it.architecture == previousInstaller.architecture &&
                        it.installerType == previousInstaller.installerType &&
                        it.scope == previousInstaller.scope
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.architecture == previousInstaller.architecture &&
                            it.installerType == previousInstaller.installerType &&
                            it.scope == null
                }
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.architecture == previousInstaller.architecture && it.scope == previousInstaller.scope
                }
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.architecture == previousInstaller.architecture &&
                            it.installerType == previousInstaller.installerType
                }
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.installerType == previousInstaller.installerType
                }
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.architecture == previousInstaller.architecture
                }
            }
            if (newInstaller == null) {
                newInstaller = newInstallers.firstOrNull {
                    it.installerUrl.getExtension() == previousInstaller.installerUrl.getExtension()
                }
            }
            if (newInstaller != null) {
                result[previousInstaller] = newInstaller
            }
        }
        return result
    }
}
