package detection

import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.http.Url
import network.HttpUtils
import org.koin.core.component.KoinComponent
import schemas.manifest.InstallerManifest

object ParameterUrls : KoinComponent {
    fun assertUniqueUrlsCount(parameterUrls: List<Url>, previousUrls: List<Url>, terminal: Terminal) {
        if (parameterUrls.distinct().size != previousUrls.distinct().size) {
            throw CliktError(
                terminal.colors.danger(
                    buildString {
                        append("The number of unique installer urls ")
                        append(
                            when {
                                parameterUrls.distinct().size > previousUrls.distinct().size -> "is greater than"
                                parameterUrls.distinct().size < previousUrls.distinct().size -> "is less than"
                                else -> "does not match"
                            }
                        )
                        append(" the number of previous manifest urls")
                    }
                )
            )
        }
    }

    suspend fun assertUrlsValid(parameterUrls: List<Url>, terminal: Terminal) {
        parameterUrls.forEach { url ->
            data.shared.Url.isUrlValid(url, false)
                ?.let { throw CliktError(terminal.colors.danger("$it on $url")) }
        }
    }

    fun matchInstallers(
        newInstallers: List<InstallerManifest.Installer>,
        previousInstallers: List<InstallerManifest.Installer>
    ): List<Pair<InstallerManifest.Installer, InstallerManifest.Installer>> {
        val result = mutableListOf<Pair<InstallerManifest.Installer, InstallerManifest.Installer>>()
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
                    HttpUtils.getURLExtension(it.installerUrl) ==
                        HttpUtils.getURLExtension(previousInstaller.installerUrl)
                }
            }
            if (newInstaller != null) {
                result.add(Pair(previousInstaller, newInstaller))
            }
        }
        return result
    }
}
