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
    val packageIdentifier: String,
    val packageVersion: String,
    val channel: String? = null,
    val installerLocale: String? = null,
    val platform: List<Platform>? = null,
    val minimumOSVersion: String? = null,
    val installerType: InstallerType? = null,
    val nestedInstallerType: NestedInstallerType? = null,
    val nestedInstallerFiles: List<NestedInstallerFiles>? = null,
    val scope: Scope? = null,
    val installModes: List<InstallModes>? = null,
    val installerSwitches: InstallerSwitches? = null,
    val installerSuccessCodes: List<Long>? = null,
    val expectedReturnCodes: List<ExpectedReturnCodes>? = null,
    val upgradeBehavior: UpgradeBehavior? = null,
    val commands: List<String>? = null,
    val protocols: List<String>? = null,
    val fileExtensions: List<String>? = null,
    val dependencies: Dependencies? = null,
    val packageFamilyName: String? = null,
    val productCode: String? = null,
    val capabilities: List<String>? = null,
    val restrictedCapabilities: List<String>? = null,
    @Contextual val markets: Any? = null,
    val installerAbortsTerminal: Boolean? = null,
    val releaseDate: LocalDate? = null,
    val installLocationRequired: Boolean? = null,
    val requireExplicitUpgrade: Boolean? = null,
    val displayInstallWarnings: Boolean? = null,
    val unsupportedOSArchitectures: List<UnsupportedOSArchitectures>? = null,
    val unsupportedArguments: List<UnsupportedArguments>? = null,
    val appsAndFeaturesEntries: List<AppsAndFeaturesEntry>? = null,
    val elevationRequirement: ElevationRequirement? = null,
    val installationMetadata: InstallationMetadata? = null,
    val installers: List<Installer> = emptyList(),
    val manifestType: String,
    val manifestVersion: String
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
        val relativeFilePath: String,
        val portableCommandAlias: String? = null
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
        var silent: String? = null,
        var silentWithProgress: String? = null,
        val interactive: String? = null,
        val installLocation: String? = null,
        val log: String? = null,
        val upgrade: String? = null,
        var custom: String? = null
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
        val installerReturnCode: Int? = null,
        val returnResponse: ReturnResponse,
        val returnResponseUrl: String? = null
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
        val windowsFeatures: List<String>? = null,
        val windowsLibraries: List<String>? = null,
        val packageDependencies: List<PackageDependencies>? = null,
        val externalDependencies: List<String>? = null
    ) {
        @Serializable
        data class PackageDependencies(
            val packageIdentifier: String,
            val minimumVersion: String? = null
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
        val displayName: String? = null,
        val publisher: String? = null,
        val displayVersion: String? = null,
        val productCode: String? = null,
        val upgradeCode: String? = null,
        val installerType: InstallerType? = null
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
        val defaultInstallLocation: String? = null,
        val files: List<Files>? = null
    ) {
        /**
         * Represents an installed file.
         */
        @Serializable
        data class Files(
            val relativeFilePath: String,
            val fileSha256: String? = null,
            val fileType: FileType? = null,
            val invocationParameter: String? = null,
            val displayName: String? = null
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
        val installerLocale: String? = null,
        val platform: List<Platform>? = null,
        val minimumOSVersion: String? = null,
        val architecture: Architecture,
        val installerType: InstallerType? = null,
        val nestedInstallerType: NestedInstallerType? = null,
        val nestedInstallerFiles: List<NestedInstallerFiles>? = null,
        val scope: Scope? = null,
        @Contextual val installerUrl: Url,
        val installerSha256: String,
        val signatureSha256: String? = null,
        val installModes: List<InstallModes>? = null,
        val installerSwitches: InstallerSwitches? = null,
        val installerSuccessCodes: List<Long>? = null,
        val expectedReturnCodes: List<ExpectedReturnCodes>? = null,
        val upgradeBehavior: UpgradeBehavior? = null,
        val commands: List<String>? = null,
        val protocols: List<String>? = null,
        val fileExtensions: List<String>? = null,
        val dependencies: Dependencies? = null,
        val packageFamilyName: String? = null,
        val productCode: String? = null,
        val capabilities: List<String>? = null,
        val restrictedCapabilities: List<String>? = null,
        @Contextual val markets: Any? = null,
        val installerAbortsTerminal: Boolean? = null,
        val releaseDate: LocalDate? = null,
        val installLocationRequired: Boolean? = null,
        val requireExplicitUpgrade: Boolean? = null,
        val displayInstallWarnings: Boolean? = null,
        val unsupportedOSArchitectures: List<UnsupportedOSArchitectures>? = null,
        val unsupportedArguments: List<UnsupportedArguments>? = null,
        val appsAndFeaturesEntries: List<AppsAndFeaturesEntry>? = null,
        val elevationRequirement: ElevationRequirement? = null,
        val installationMetadata: InstallationMetadata? = null
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
