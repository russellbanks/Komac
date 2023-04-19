package detection

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.TerminalColors
import io.ktor.client.HttpClient
import io.ktor.http.Url
import network.Http
import schemas.manifest.InstallerManifest
import utils.extension

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

    suspend fun assertUrlsValid(parameterUrls: Set<Url>, colors: TerminalColors) {
        val errorList = parameterUrls.mapNotNull { url ->
            data.shared.Url.isUrlValid(url, false, Http.client)?.let { error -> url to error }
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
        val result = mutableMapOf<InstallerManifest.Installer, InstallerManifest.Installer>()

        for (previousInstaller in previousInstallers) {
            val matchingConditions = sequenceOf<(InstallerManifest.Installer) -> Boolean>(
                { it.architecture == previousInstaller.architecture &&
                        it.installerType == previousInstaller.installerType &&
                        it.scope == previousInstaller.scope },
                { it.architecture == previousInstaller.architecture
                        && it.installerType == previousInstaller.installerType
                        && it.scope == null },
                { it.architecture == previousInstaller.architecture &&
                        it.installerType == null &&
                        it.scope == previousInstaller.scope },
                { it.architecture == previousInstaller.architecture &&
                        it.installerType == previousInstaller.installerType },
                { it.installerType == previousInstaller.installerType },
                { it.architecture == previousInstaller.architecture },
                { it.installerUrl.extension == previousInstaller.installerUrl.extension }
            )

            val newInstaller = matchingConditions
                .mapNotNull(newInstallers::firstOrNull)
                .firstOrNull()

            newInstaller?.let { result[previousInstaller] = it }
        }

        return result
    }
}
