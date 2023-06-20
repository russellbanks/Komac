package schemas.manifest

import data.InstallerManifestData
import data.PreviousManifestData
import github.GitHubDetection
import io.ktor.http.Url
import network.WebPageScraper
import schemas.AdditionalMetadata
import schemas.Schemas

sealed class Manifest {
    companion object {
        suspend fun createFiles(
            packageIdentifier: String,
            packageVersion: String,
            defaultLocale: String?,
            license: String,
            licenseUrl: Url? = null,
            author: String? = null,
            publisher: String,
            publisherUrl: Url? = null,
            packageUrl: Url? = null,
            copyright: String? = null,
            copyrightUrl: Url? = null,
            shortDescription: String,
            moniker: String? = null,
            allVersions: List<String>? = null,
            installers: List<InstallerManifest.Installer>,
            manifestOverride: String,
            packageName: String,
            previousManifestData: PreviousManifestData?,
            additionalMetadata: AdditionalMetadata? = null,
            gitHubDetection: GitHubDetection?,
            pageScraper: WebPageScraper?
        ): Map<String, Manifest> {
            val allLocale = additionalMetadata?.locales?.find { it.name.equals("all", ignoreCase = true) }
            val previousInstallerManifest = previousManifestData?.installerManifest?.await()
            return mapOf(
                InstallerManifest.getFileName(packageIdentifier) to InstallerManifestData.createInstallerManifest(
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    allVersions = allVersions,
                    installers = installers,
                    previousInstallerManifest = previousInstallerManifest,
                    manifestOverride = manifestOverride
                ),
                DefaultLocaleManifest.getFileName(
                    identifier = packageIdentifier,
                    defaultLocale = defaultLocale,
                    previousDefaultLocale = previousManifestData?.versionManifest?.defaultLocale
                ) to DefaultLocaleManifest.create(
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    defaultLocale = defaultLocale,
                    license = license,
                    licenseUrl = licenseUrl,
                    author = author,
                    publisherUrl = publisherUrl,
                    packageUrl = packageUrl,
                    copyright = copyright,
                    copyrightUrl = copyrightUrl,
                    shortDescription = shortDescription,
                    moniker = moniker,
                    publisher = publisher,
                    packageName = packageName,
                    previousManifestData = previousManifestData,
                    gitHubDetection = gitHubDetection,
                    pageScraper = pageScraper,
                    manifestOverride = manifestOverride
                ),
                VersionManifest.getFileName(packageIdentifier) to VersionManifest.create(
                    packageIdentifier = packageIdentifier,
                    packageVersion = packageVersion,
                    defaultLocale = defaultLocale,
                    previousDefaultLocale = previousManifestData?.versionManifest?.defaultLocale,
                    manifestOverride = manifestOverride,
                )
            ) + previousManifestData?.localeManifests?.map { localeManifest ->
                val currentLocaleMetadata = additionalMetadata?.locales
                    ?.find { it.name.equals(localeManifest.packageLocale, ignoreCase = true) }

                LocaleManifest.getFileName(
                    identifier = packageIdentifier,
                    locale = localeManifest.packageLocale
                ) to if (allLocale != null || currentLocaleMetadata != null) {
                    localeManifest.copy(
                        packageIdentifier = packageIdentifier,
                        packageVersion = packageVersion,
                        manifestVersion = manifestOverride,
                        releaseNotes = allLocale?.releaseNotes ?: currentLocaleMetadata?.releaseNotes,
                        releaseNotesUrl = allLocale?.releaseNotesUrl ?: currentLocaleMetadata?.releaseNotesUrl,
                        documentations = allLocale?.documentations
                            ?: currentLocaleMetadata?.documentations
                            ?: localeManifest.documentations
                    )
                } else {
                    localeManifest.copy(
                        packageIdentifier = packageIdentifier,
                        packageVersion = packageVersion,
                        manifestVersion = Schemas.manifestVersion
                    )
                }
            }.orEmpty()
        }
    }
}
