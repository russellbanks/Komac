package schemas

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class InstallerManifest(
    @SerialName("PackageIdentifier") val packageIdentifier: String? = null,
    @SerialName("PackageVersion") val packageVersion: String? = null,
    @SerialName("Channel") val channel: String? = null,
    @SerialName("Commands") val commands: List<String>? = null,
    @SerialName("Protocols") val protocols: List<String>? = null,
    @SerialName("FileExtensions") val fileExtensions: List<String>? = null,
    @SerialName("Installers") val installers: List<Installer> = listOf(),
    @SerialName("ManifestType") val manifestType: String = "",
    @SerialName("ManifestVersion") val manifestVersion: String = ""
) {
    @Serializable
    data class Installer(
        @SerialName("Architecture") val architecture: String? = null,
        @SerialName("InstallerLocale") val installerLocale: String? = null,
        @SerialName("Platform") val platform: String? = null,
        @SerialName("MinimumOSVersion") val minimumOSVersion: String? = null,
        @SerialName("InstallerType") val installerType: String? = null,
        @SerialName("InstallerUrl") val installerUrl: String? = null,
        @SerialName("InstallerSha256") val installerSha256: String? = null,
        @SerialName("SignatureSha256") val signatureSha256: String? = null,
        @SerialName("Scope") val scope: String? = null,
        @SerialName("InstallModes") val installModes: String? = null,
        @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches = InstallerSwitches(),
        @SerialName("UpgradeBehavior") val upgradeBehavior: String? = null,
        @SerialName("Dependencies") val dependencies: List<Dependency> = listOf(),
        @SerialName("PackageFamilyName") val packageFamilyName: String? = null,
        @SerialName("Capabilities") val capabilities: String? = null,
        @SerialName("RestrictedCapabilities") val restrictedCapabilities: String? = null,
        @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: String? = null,
        @SerialName("InstallLocationRequired") val installLocationRequired: String? = null,
        @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: String? = null,
        @SerialName("ElevationRequirement") val elevationRequirement: String? = null,
        @SerialName("UnsupportedOSArchitectures") val unsupportedOSArchitectures: String? = null,
        @SerialName("Markets") val markets: String? = null,
        @SerialName("ExcludedMarkets") val excludedMarkets: String? = null,
        @SerialName("InstallerSuccessCodes") val installerSuccessCodes: String? = null,
        @SerialName("ExpectedReturnCodes") val expectedReturnCodes: List<ExpectedReturnCode> = listOf(),
        @SerialName("ProductCode") val productCode: String? = null,
        @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: List<AppsAndFeaturesEntry> = listOf(),
        @SerialName("UnsupportedArguments") val unsupportedArguments: List<UnsupportedArgument> = listOf(),
        @SerialName("DisplayInstallWarnings") val displayInstallWarnings: String? = null,
        @SerialName("ReleaseDate") val releaseDate: String? = null
    ) {
        @Serializable
        data class InstallerSwitches(
            @SerialName("Silent") val silent: String? = null,
            @SerialName("SilentWithProgress") val silentWithProgress: String? = null,
            @SerialName("Interactive") val interactive: String? = null,
            @SerialName("InstallLocation") val installLocation: String? = null,
            @SerialName("Log") val log: String? = null,
            @SerialName("Upgrade") val upgrade: String? = null,
            @SerialName("Custom") val custom: String? = null
        )

        @Serializable
        data class Dependency(
            @SerialName("ExternalDependencies") val externalDependencies: String? = null,
            @SerialName("PackageDependencies") val packageDependencies: String? = null,
            @SerialName("WindowsFeatures") val windowsFeatures: String? = null,
            @SerialName("WindowsLibraries") val windowsLibraries: String? = null
        )

        @Serializable
        data class ExpectedReturnCode(
            @SerialName("ExpectedReturnCode") val expectedReturnCode: String? = null,
            @SerialName("ReturnResponse") val returnResponse: String? = null,
            @SerialName("ReturnResponseUrl") val returnResponseUrl: String? = null
        )

        @Serializable
        data class AppsAndFeaturesEntry(
            @SerialName("DisplayName") val displayName: String? = null,
            @SerialName("DisplayVersion") val displayVersion: String? = null,
            @SerialName("Publisher") val publisher: String? = null,
            @SerialName("ProductCode") val productCode: String? = null,
            @SerialName("UpgradeCode") val upgradeCode: String? = null,
            @SerialName("InstallerType") val installerType: String? = null
        )

        @Serializable
        data class UnsupportedArgument(
            @SerialName("UnsupportedArgument") val unsupportedArgument: String? = null
        )
    }
}
