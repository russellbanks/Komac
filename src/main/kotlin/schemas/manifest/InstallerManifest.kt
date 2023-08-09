package schemas.manifest

import io.ktor.http.Url
import kotlinx.datetime.LocalDate
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import schemas.SchemaType
import schemas.Schemas

/**
 * A representation of a single-file manifest representing an app installers in the OWC. v1.5.0
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
    val nestedInstallerType: InstallerType? = null,
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
    val markets: Markets? = null,
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
    val manifestType: String = SchemaType.INSTALLER,
    val manifestVersion: String = Schemas.MANIFEST_VERSION
) : Manifest() {
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

        companion object {
            const val MSIXBUNDLE = "msixbundle"
            const val APPXBUNDLE = "appxbundle"
            fun fileExtensions() = listOf(MSIX.name, MSI.name, APPX.name, EXE.name, ZIP.name, MSIXBUNDLE, APPXBUNDLE)
                .map(String::lowercase)
        }
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
        var interactive: String? = null,
        var installLocation: String? = null,
        var log: String? = null,
        var upgrade: String? = null,
        var custom: String? = null
    ) {
        enum class Key {
            Silent, SilentWithProgress, Interactive, InstallLocation, Log, Upgrade, Custom
        }

        operator fun get(key: Key): String? = when (key) {
            Key.Silent -> silent
            Key.SilentWithProgress -> silentWithProgress
            Key.Interactive -> interactive
            Key.InstallLocation -> installLocation
            Key.Log -> log
            Key.Upgrade -> upgrade
            Key.Custom -> custom
        }

        operator fun set(key: Key, value: String) {
            when (key) {
                Key.Silent -> silent = value
                Key.SilentWithProgress -> silentWithProgress = value
                Key.Interactive -> interactive = value
                Key.InstallLocation -> installLocation = value
                Key.Log -> log = value
                Key.Upgrade -> upgrade = value
                Key.Custom -> custom = value
            }
        }

        fun areAllNull(): Boolean = Key.entries.all { get(it) == null }

        fun areAllNullOrBlank(): Boolean = Key.entries.all { get(it).isNullOrBlank() }
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

    @Serializable
    data class Markets(
        val allowedMarkets: List<String>? = null,
        val excludedMarkets: List<String>? = null
    )

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
        val nestedInstallerType: InstallerType? = null,
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
        val markets: Markets? = null,
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

            companion object {
                fun valueOfOrNull(value: String): Architecture? = try {
                    valueOf(value)
                } catch (_: IllegalArgumentException) {
                    null
                }
            }
        }
    }

    override fun toString() = Schemas.buildManifestString(
        manifest = this,
        rawString = EncodeConfig.yamlDefault.encodeToString(serializer = serializer(), value = this)
    )

    companion object {
        /**
         * Returns the name of the YAML file containing the installer manifest for the given package identifier.
         *
         * @param identifier the package identifier to get the installer manifest name for
         * @return a string representing the name of the YAML file containing the installer manifest for the given
         * identifier
         */
        fun getFileName(identifier: String) = "$identifier.installer.yaml"

        fun getBase(
            previousManifest: InstallerManifest?,
            packageIdentifier: String,
            packageVersion: String
        ): InstallerManifest = previousManifest ?: InstallerManifest(
            packageIdentifier = packageIdentifier,
            packageVersion = packageVersion,
            manifestType = SchemaType.INSTALLER,
            manifestVersion = Schemas.MANIFEST_VERSION
        )

        fun getInstallerBase(previousInstaller: Installer?): Installer = previousInstaller ?: Installer(
            architecture = Installer.Architecture.NEUTRAL,
            installerSha256 = "",
            installerUrl = Url("")
        )
    }
}
