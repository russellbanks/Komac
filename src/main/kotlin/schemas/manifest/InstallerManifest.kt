package schemas.manifest

import input.Switch
import io.ktor.http.Url
import kotlinx.datetime.LocalDate
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.Schema
import schemas.Schemas

/**
 * A representation of a single-file manifest representing an app installers in the OWC. v1.4.0
 */

@Suppress("unused")
@Serializable
data class InstallerManifest(
    @SerialName("PackageIdentifier") val packageIdentifier: String,
    @SerialName("PackageVersion") val packageVersion: String,
    @SerialName("Channel") val channel: String? = null,
    @SerialName("InstallerLocale") val installerLocale: String? = null,
    @SerialName("Platform") val platform: List<Platform>? = null,
    @SerialName("MinimumOSVersion") val minimumOSVersion: String? = null,
    @SerialName("InstallerType") val installerType: InstallerType? = null,
    @SerialName("NestedInstallerType") val nestedInstallerType: NestedInstallerType? = null,
    @SerialName("NestedInstallerFiles") val nestedInstallerFiles: List<NestedInstallerFiles>? = null,
    @SerialName("Scope") val scope: Scope? = null,
    @SerialName("InstallModes") val installModes: List<InstallModes>? = null,
    @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches? = null,
    @SerialName("InstallerSuccessCodes") val installerSuccessCodes: List<Long>? = null,
    @SerialName("ExpectedReturnCodes") val expectedReturnCodes: List<ExpectedReturnCodes>? = null,
    @SerialName("UpgradeBehavior") val upgradeBehavior: UpgradeBehavior? = null,
    @SerialName("Commands") val commands: List<String>? = null,
    @SerialName("Protocols") val protocols: List<String>? = null,
    @SerialName("FileExtensions") val fileExtensions: List<String>? = null,
    @SerialName("Dependencies") val dependencies: Dependencies? = null,
    @SerialName("PackageFamilyName") val packageFamilyName: String? = null,
    @SerialName("ProductCode") val productCode: String? = null,
    @SerialName("Capabilities") val capabilities: List<String>? = null,
    @SerialName("RestrictedCapabilities") val restrictedCapabilities: List<String>? = null,
    @SerialName("Markets") @Contextual val markets: Any? = null,
    @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: Boolean? = null,
    @SerialName("ReleaseDate") val releaseDate: LocalDate? = null,
    @SerialName("InstallLocationRequired") val installLocationRequired: Boolean? = null,
    @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: Boolean? = null,
    @SerialName("DisplayInstallWarnings") val displayInstallWarnings: Boolean? = null,
    @SerialName("UnsupportedOSArchitectures") val unsupportedOSArchitectures: List<UnsupportedOSArchitectures>? = null,
    @SerialName("UnsupportedArguments") val unsupportedArguments: List<UnsupportedArguments>? = null,
    @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: List<AppsAndFeaturesEntry>? = null,
    @SerialName("ElevationRequirement") val elevationRequirement: ElevationRequirement? = null,
    @SerialName("InstallationMetadata") val installationMetadata: InstallationMetadata? = null,
    @SerialName("Installers") val installers: List<Installer> = emptyList(),
    @SerialName("ManifestType") val manifestType: String,
    @SerialName("ManifestVersion") val manifestVersion: String
) {
    enum class Platform {
        @SerialName("Windows.Desktop") WindowsDesktop,
        @SerialName("Windows.Universal") WindowsUniversal;

        override fun toString() = name.split("(?<=[a-z])(?=[A-Z])".toRegex()).joinToString(".")
    }

    /**
     * Enumeration of supported installer types.
     * InstallerType is required in either root level or individual Installer level
     */
    enum class InstallerType {
        @SerialName("msix") MSIX,
        @SerialName("msi") MSI,
        @SerialName("appx") APPX,
        @SerialName("exe") EXE,
        @SerialName("zip") ZIP,
        @SerialName("inno") INNO,
        @SerialName("nullsoft") NULLSOFT,
        @SerialName("wix") WIX,
        @SerialName("burn") BURN,
        @SerialName("pwa") PWA,
        @SerialName("portable") PORTABLE;

        override fun toString() = name.lowercase()
    }

    /**
     * Enumeration of supported nested installer types contained inside an archive file
     */
    enum class NestedInstallerType {
        @SerialName("msix") MSIX,
        @SerialName("msi") MSI,
        @SerialName("appx") APPX,
        @SerialName("exe") EXE,
        @SerialName("zip") ZIP,
        @SerialName("inno") INNO,
        @SerialName("nullsoft") NULLSOFT,
        @SerialName("wix") WIX,
        @SerialName("burn") BURN,
        @SerialName("portable") PORTABLE;

        override fun toString() = name.lowercase()
    }

    /**
     * A nested installer file contained inside an archive
     */
    @Serializable
    data class NestedInstallerFiles(
        @SerialName("RelativeFilePath") val relativeFilePath: String,
        @SerialName("PortableCommandAlias") val portableCommandAlias: String? = null
    )

    /**
     * Scope indicates if the installer is per user or per machine
     */
    enum class Scope {
        @SerialName("user") User,
        @SerialName("machine") Machine
    }

    enum class InstallModes {
        @SerialName("interactive") Interactive,
        @SerialName("silent") Silent,
        @SerialName("silentWithProgress") SilentWithProgress
    }

    @Serializable
    data class InstallerSwitches(
        @SerialName("Silent") var silent: String? = null,
        @SerialName("SilentWithProgress") var silentWithProgress: String? = null,
        @SerialName("Interactive") val interactive: String? = null,
        @SerialName("InstallLocation") val installLocation: String? = null,
        @SerialName("Log") val log: String? = null,
        @SerialName("Upgrade") val upgrade: String? = null,
        @SerialName("Custom") var custom: String? = null
    ) {
        private val listOfAll = listOf(silent, silentWithProgress, interactive, installLocation, log, upgrade, custom)
        fun areAllNull(): Boolean = listOfAll.all { it == null }

        fun areAllNullOrBlank(): Boolean = listOfAll.all(String?::isNullOrBlank)

        operator fun set(aSwitch: Switch, value: String?) {
            when (aSwitch) {
                Switch.Silent -> silent = value
                Switch.SilentWithProgress -> silentWithProgress = value
                Switch.Custom -> custom = value
            }
        }
    }

    @Serializable
    data class ExpectedReturnCodes(
        @SerialName("InstallerReturnCode") val installerReturnCode: Int? = null,
        @SerialName("ReturnResponse") val returnResponse: ReturnResponse,
        @SerialName("ReturnResponseUrl") val returnResponseUrl: String? = null
    ) {
        enum class ReturnResponse {
            @SerialName("packageInUse") PackageInUse,
            @SerialName("packageInUseByApplication") PackageInUseByApplication,
            @SerialName("installInProgress") InstallInProgress,
            @SerialName("fileInUse") FileInUse,
            @SerialName("missingDependency") MissingDependency,
            @SerialName("diskFull") DiskFull,
            @SerialName("insufficientMemory") InsufficientMemory,
            @SerialName("invalidParameter") InvalidParameter,
            @SerialName("noNetwork") NoNetwork,
            @SerialName("contactSupport") ContactSupport,
            @SerialName("rebootRequiredToFinish") RebootRequiredToFinish,
            @SerialName("rebootRequiredForInstall") RebootRequiredForInstall,
            @SerialName("rebootInitiated") RebootInitiated,
            @SerialName("cancelledByUser") CancelledByUser,
            @SerialName("alreadyInstalled") AlreadyInstalled,
            @SerialName("downgrade") Downgrade,
            @SerialName("blockedByPolicy") BlockedByPolicy,
            @SerialName("systemNotSupported") SystemNotSupported,
            @SerialName("custom") Custom
        }
    }

    /**
     * The upgrade method
     */
    enum class UpgradeBehavior {
        @SerialName("install") Install,
        @SerialName("uninstallPrevious") UninstallPrevious
    }

    @Serializable
    data class Dependencies(
        @SerialName("WindowsFeatures") val windowsFeatures: List<String>? = null,
        @SerialName("WindowsLibraries") val windowsLibraries: List<String>? = null,
        @SerialName("PackageDependencies") val packageDependencies: List<PackageDependencies>? = null,
        @SerialName("ExternalDependencies") val externalDependencies: List<String>? = null
    ) {
        @Serializable
        data class PackageDependencies(
            @SerialName("PackageIdentifier") val packageIdentifier: String,
            @SerialName("MinimumVersion") val minimumVersion: String? = null
        )
    }

    enum class UnsupportedOSArchitectures {
        @SerialName("x86") X86,
        @SerialName("x64") X64,
        @SerialName("arm") ARM,
        @SerialName("arm64") ARM64
    }

    enum class UnsupportedArguments {
        @SerialName("log") Log,
        @SerialName("location") Location
    }

    /**
     * Various key values under installer's ARP entry
     */
    @Serializable
    data class AppsAndFeaturesEntry(
        @SerialName("DisplayName") val displayName: String? = null,
        @SerialName("Publisher") val publisher: String? = null,
        @SerialName("DisplayVersion") val displayVersion: String? = null,
        @SerialName("ProductCode") val productCode: String? = null,
        @SerialName("UpgradeCode") val upgradeCode: String? = null,
        @SerialName("InstallerType") val installerType: InstallerType? = null
    ) {
        fun areAllNull(): Boolean {
            return listOf(displayName, publisher, displayVersion, productCode, upgradeCode, installerType).all {
                it == null
            }
        }

        /**
         * Enumeration of supported installer types.
         * InstallerType is required in either root level or individual Installer level
         */
        enum class InstallerType {
            @SerialName("msix") MSIX,
            @SerialName("msi") MSI,
            @SerialName("appx") APPX,
            @SerialName("exe") EXE,
            @SerialName("zip") ZIP,
            @SerialName("inno") INNO,
            @SerialName("nullsoft") NULLSOFT,
            @SerialName("wix") WIX,
            @SerialName("burn") BURN,
            @SerialName("pwa") PWA,
            @SerialName("portable") PORTABLE;

            override fun toString() = name.lowercase()
        }
    }

    /**
     * The installer's elevation requirement
     */
    enum class ElevationRequirement {
        @SerialName("elevationRequired") ElevationRequired,
        @SerialName("elevationProhibited") ElevationProhibited,
        @SerialName("elevatesSelf") ElevatesSelf
    }

    /**
     * Details about the installation. Used for deeper installation detection.
     */
    @Serializable
    data class InstallationMetadata(
        @SerialName("DefaultInstallLocation") val defaultInstallLocation: String? = null,
        @SerialName("Files") val files: List<Files>? = null
    ) {
        /**
         * Represents an installed file.
         */
        @Serializable
        data class Files(
            @SerialName("RelativeFilePath") val relativeFilePath: String,
            @SerialName("FileSha256") val fileSha256: String? = null,
            @SerialName("FileType") val fileType: FileType? = null,
            @SerialName("InvocationParameter") val invocationParameter: String? = null,
            @SerialName("DisplayName") val displayName: String? = null
        ) {
            /**
             * The optional installed file type. If not specified, the file is treated as other.
             */
            enum class FileType {
                @SerialName("launch") Launch,
                @SerialName("uninstall") Uninstall,
                @SerialName("other") Other
            }
        }
    }

    @Serializable
    data class Installer(
        @SerialName("InstallerLocale") val installerLocale: String? = null,
        @SerialName("Platform") val platform: List<Platform>? = null,
        @SerialName("MinimumOSVersion") val minimumOSVersion: String? = null,
        @SerialName("Architecture") val architecture: Architecture,
        @SerialName("InstallerType") val installerType: InstallerType? = null,
        @SerialName("NestedInstallerType") val nestedInstallerType: NestedInstallerType? = null,
        @SerialName("NestedInstallerFiles") val nestedInstallerFiles: List<NestedInstallerFiles>? = null,
        @SerialName("Scope") val scope: Scope? = null,
        @SerialName("InstallerUrl") @Contextual val installerUrl: Url,
        @SerialName("InstallerSha256") val installerSha256: String,
        @SerialName("SignatureSha256") val signatureSha256: String? = null,
        @SerialName("InstallModes") val installModes: List<InstallModes>? = null,
        @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches? = null,
        @SerialName("InstallerSuccessCodes") val installerSuccessCodes: List<Long>? = null,
        @SerialName("ExpectedReturnCodes") val expectedReturnCodes: List<ExpectedReturnCodes>? = null,
        @SerialName("UpgradeBehavior") val upgradeBehavior: UpgradeBehavior? = null,
        @SerialName("Commands") val commands: List<String>? = null,
        @SerialName("Protocols") val protocols: List<String>? = null,
        @SerialName("FileExtensions") val fileExtensions: List<String>? = null,
        @SerialName("Dependencies") val dependencies: Dependencies? = null,
        @SerialName("PackageFamilyName") val packageFamilyName: String? = null,
        @SerialName("ProductCode") val productCode: String? = null,
        @SerialName("Capabilities") val capabilities: List<String>? = null,
        @SerialName("RestrictedCapabilities") val restrictedCapabilities: List<String>? = null,
        @SerialName("Markets") @Contextual val markets: Any? = null,
        @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: Boolean? = null,
        @SerialName("ReleaseDate") val releaseDate: LocalDate? = null,
        @SerialName("InstallLocationRequired") val installLocationRequired: Boolean? = null,
        @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: Boolean? = null,
        @SerialName("DisplayInstallWarnings") val displayInstallWarnings: Boolean? = null,
        @SerialName("UnsupportedOSArchitectures")
        val unsupportedOSArchitectures: List<UnsupportedOSArchitectures>? = null,
        @SerialName("UnsupportedArguments") val unsupportedArguments: List<UnsupportedArguments>? = null,
        @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: List<AppsAndFeaturesEntry>? = null,
        @SerialName("ElevationRequirement") val elevationRequirement: ElevationRequirement? = null,
        @SerialName("InstallationMetadata") val installationMetadata: InstallationMetadata? = null
    ) {
        /**
         * The installer target architecture
         */
        enum class Architecture {
            @SerialName("x86") X86,
            @SerialName("x64") X64,
            @SerialName("arm") ARM,
            @SerialName("arm64") ARM64,
            @SerialName("neutral") NEUTRAL;

            override fun toString() = name.lowercase()
        }
    }

    override fun toString() = Schemas.buildManifestString(
        schema = Schema.Installer,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )
}
