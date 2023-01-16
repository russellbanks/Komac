package schemas.data

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class InstallerSchema(
    @SerialName("\$id") val id: String,
    @SerialName("\$schema") val schema: String,
    @SerialName("description") val description: String,
    @SerialName("definitions") val definitions: Definitions,
    @SerialName("type") val type: String,
    @SerialName("properties") val properties: Properties,
    @SerialName("required") val required: List<String>
) : RemoteSchema {
    @Serializable
    data class Definitions(
        @SerialName("PackageIdentifier") val packageIdentifier: PackageIdentifier,
        @SerialName("PackageVersion") val packageVersion: PackageVersion,
        @SerialName("Locale") val locale: Locale,
        @SerialName("Channel") val channel: Channel,
        @SerialName("Platform") val platform: Platform,
        @SerialName("MinimumOSVersion") val minimumOSVersion: MinimumOSVersion,
        @SerialName("Url") val url: Url,
        @SerialName("InstallerType") val installerType: InstallerType,
        @SerialName("NestedInstallerType") val nestedInstallerType: NestedInstallerType,
        @SerialName("NestedInstallerFiles") val nestedInstallerFiles: NestedInstallerFiles,
        @SerialName("Architecture") val architecture: Architecture,
        @SerialName("Scope") val scope: Scope,
        @SerialName("InstallModes") val installModes: InstallModes,
        @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches,
        @SerialName("InstallerReturnCode") val installerReturnCode: InstallerReturnCode,
        @SerialName("InstallerSuccessCodes") val installerSuccessCodes: InstallerSuccessCodes,
        @SerialName("ExpectedReturnCodes") val expectedReturnCodes: ExpectedReturnCodes,
        @SerialName("UpgradeBehavior") val upgradeBehavior: UpgradeBehavior,
        @SerialName("Commands") val commands: Commands,
        @SerialName("Protocols") val protocols: Protocols,
        @SerialName("FileExtensions") val fileExtensions: FileExtensions,
        @SerialName("Dependencies") val dependencies: Dependencies,
        @SerialName("PackageFamilyName") val packageFamilyName: PackageFamilyName,
        @SerialName("ProductCode") val productCode: ProductCode,
        @SerialName("Capabilities") val capabilities: Capabilities,
        @SerialName("RestrictedCapabilities") val restrictedCapabilities: RestrictedCapabilities,
        @SerialName("Market") val market: Market,
        @SerialName("MarketArray") val marketArray: MarketArray,
        @SerialName("Markets") val markets: Markets,
        @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: InstallerAbortsTerminal,
        @SerialName("ReleaseDate") val releaseDate: ReleaseDate,
        @SerialName("InstallLocationRequired") val installLocationRequired: InstallLocationRequired,
        @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: RequireExplicitUpgrade,
        @SerialName("DisplayInstallWarnings") val displayInstallWarnings: DisplayInstallWarnings,
        @SerialName("UnsupportedOSArchitectures") val unsupportedOSArchitectures: UnsupportedOSArchitectures,
        @SerialName("UnsupportedArguments") val unsupportedArguments: UnsupportedArguments,
        @SerialName("AppsAndFeaturesEntry") val appsAndFeaturesEntry: AppsAndFeaturesEntry,
        @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: AppsAndFeaturesEntries,
        @SerialName("ElevationRequirement") val elevationRequirement: ElevationRequirement,
        @SerialName("InstallationMetadata") val installationMetadata: InstallationMetadata,
        @SerialName("Installer") val installer: Installer
    ) {
        @Serializable
        data class PackageIdentifier(
            @SerialName("type") val type: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class PackageVersion(
            @SerialName("type") val type: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Locale(
            @SerialName("type") val type: List<String>,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Channel(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Platform(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("title") val title: String,
                @SerialName("type") val type: String,
                @SerialName("enum") val enum: List<String>
            )
        }

        @Serializable
        data class MinimumOSVersion(
            @SerialName("type") val type: List<String>,
            @SerialName("pattern") val pattern: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Url(
            @SerialName("type") val type: List<String>,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class InstallerType(
            @SerialName("type") val type: List<String>,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class NestedInstallerType(
            @SerialName("type") val type: List<String>,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class NestedInstallerFiles(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("title") val title: String,
                @SerialName("properties") val properties: Properties,
                @SerialName("required") val required: List<String>,
                @SerialName("description") val description: String
            ) {
                @Serializable
                data class Properties(
                    @SerialName("RelativeFilePath") val relativeFilePath: RelativeFilePath,
                    @SerialName("PortableCommandAlias") val portableCommandAlias: PortableCommandAlias
                ) {
                    @Serializable
                    data class RelativeFilePath(
                        @SerialName("type") val type: String,
                        @SerialName("minLength") val minLength: Int,
                        @SerialName("maxLength") val maxLength: Int,
                        @SerialName("description") val description: String
                    )

                    @Serializable
                    data class PortableCommandAlias(
                        @SerialName("type") val type: List<String>,
                        @SerialName("minLength") val minLength: Int,
                        @SerialName("maxLength") val maxLength: Int,
                        @SerialName("description") val description: String
                    )
                }
            }
        }

        @Serializable
        data class Architecture(
            @SerialName("type") val type: String,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Scope(
            @SerialName("type") val type: List<String>,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class InstallModes(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("title") val title: String,
                @SerialName("type") val type: String,
                @SerialName("enum") val enum: List<String>
            )
        }

        @Serializable
        data class InstallerSwitches(
            @SerialName("type") val type: String,
            @SerialName("properties") val properties: Properties
        ) {
            @Serializable
            data class Properties(
                @SerialName("Silent") val silent: Silent,
                @SerialName("SilentWithProgress") val silentWithProgress: SilentWithProgress,
                @SerialName("Interactive") val interactive: Interactive,
                @SerialName("InstallLocation") val installLocation: InstallLocation,
                @SerialName("Log") val log: Log,
                @SerialName("Upgrade") val upgrade: Upgrade,
                @SerialName("Custom") val custom: Custom
            ) {
                @Serializable
                data class Silent(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class SilentWithProgress(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Interactive(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class InstallLocation(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Log(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Upgrade(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Custom(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )
            }
        }

        @Serializable
        data class InstallerReturnCode(
            @SerialName("type") val type: String,
            @SerialName("format") val format: String,
            @SerialName("not") val not: Not,
            @SerialName("minimum") val minimum: Int,
            @SerialName("maximum") val maximum: Long,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Not(
                @SerialName("enum") val enum: List<Int>
            )
        }

        @Serializable
        data class InstallerSuccessCodes(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class ExpectedReturnCodes(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("title") val title: String,
                @SerialName("properties") val properties: Properties
            ) {
                @Serializable
                data class Properties(
                    @SerialName("InstallerReturnCode") val installerReturnCode: InstallerReturnCode,
                    @SerialName("ReturnResponse") val returnResponse: ReturnResponse,
                    @SerialName("ReturnResponseUrl") val returnResponseUrl: ReturnResponseUrl
                ) {
                    @Serializable
                    data class InstallerReturnCode(
                        @SerialName("\$ref") val ref: String
                    )

                    @Serializable
                    data class ReturnResponse(
                        @SerialName("type") val type: String,
                        @SerialName("enum") val enum: List<String>
                    )

                    @Serializable
                    data class ReturnResponseUrl(
                        @SerialName("\$ref") val ref: String,
                        @SerialName("description") val description: String
                    )
                }
            }
        }

        @Serializable
        data class UpgradeBehavior(
            @SerialName("type") val type: List<String>,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Commands(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("minLength") val minLength: Int,
                @SerialName("maxLength") val maxLength: Int
            )
        }

        @Serializable
        data class Protocols(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("maxLength") val maxLength: Int
            )
        }

        @Serializable
        data class FileExtensions(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("pattern") val pattern: String,
                @SerialName("maxLength") val maxLength: Int
            )
        }

        @Serializable
        data class Dependencies(
            @SerialName("type") val type: List<String>,
            @SerialName("properties") val properties: Properties
        ) {
            @Serializable
            data class Properties(
                @SerialName("WindowsFeatures") val windowsFeatures: WindowsFeatures,
                @SerialName("WindowsLibraries") val windowsLibraries: WindowsLibraries,
                @SerialName("PackageDependencies") val packageDependencies: PackageDependencies,
                @SerialName("ExternalDependencies") val externalDependencies: ExternalDependencies
            ) {
                @Serializable
                data class WindowsFeatures(
                    @SerialName("type") val type: List<String>,
                    @SerialName("items") val items: Items,
                    @SerialName("maxItems") val maxItems: Int,
                    @SerialName("uniqueItems") val uniqueItems: Boolean,
                    @SerialName("description") val description: String
                ) {
                    @Serializable
                    data class Items(
                        @SerialName("type") val type: String,
                        @SerialName("minLength") val minLength: Int,
                        @SerialName("maxLength") val maxLength: Int
                    )
                }

                @Serializable
                data class WindowsLibraries(
                    @SerialName("type") val type: List<String>,
                    @SerialName("items") val items: Items,
                    @SerialName("maxItems") val maxItems: Int,
                    @SerialName("uniqueItems") val uniqueItems: Boolean,
                    @SerialName("description") val description: String
                ) {
                    @Serializable
                    data class Items(
                        @SerialName("type") val type: String,
                        @SerialName("minLength") val minLength: Int,
                        @SerialName("maxLength") val maxLength: Int
                    )
                }

                @Serializable
                data class PackageDependencies(
                    @SerialName("type") val type: List<String>,
                    @SerialName("items") val items: Items,
                    @SerialName("maxItems") val maxItems: Int,
                    @SerialName("uniqueItems") val uniqueItems: Boolean,
                    @SerialName("description") val description: String
                ) {
                    @Serializable
                    data class Items(
                        @SerialName("type") val type: String,
                        @SerialName("properties") val properties: Properties,
                        @SerialName("required") val required: List<String>
                    ) {
                        @Serializable
                        data class Properties(
                            @SerialName("PackageIdentifier") val packageIdentifier: PackageIdentifier,
                            @SerialName("MinimumVersion") val minimumVersion: MinimumVersion
                        ) {
                            @Serializable
                            data class PackageIdentifier(
                                @SerialName("\$ref") val ref: String
                            )

                            @Serializable
                            data class MinimumVersion(
                                @SerialName("\$ref") val ref: String
                            )
                        }
                    }
                }

                @Serializable
                data class ExternalDependencies(
                    @SerialName("type") val type: List<String>,
                    @SerialName("items") val items: Items,
                    @SerialName("maxItems") val maxItems: Int,
                    @SerialName("uniqueItems") val uniqueItems: Boolean,
                    @SerialName("description") val description: String
                ) {
                    @Serializable
                    data class Items(
                        @SerialName("type") val type: String,
                        @SerialName("minLength") val minLength: Int,
                        @SerialName("maxLength") val maxLength: Int
                    )
                }
            }
        }

        @Serializable
        data class PackageFamilyName(
            @SerialName("type") val type: List<String>,
            @SerialName("pattern") val pattern: String,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ProductCode(
            @SerialName("type") val type: List<String>,
            @SerialName("minLength") val minLength: Int,
            @SerialName("maxLength") val maxLength: Int,
            @SerialName("description") val description: String
        )

        @Serializable
        data class Capabilities(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("minLength") val minLength: Int,
                @SerialName("maxLength") val maxLength: Int
            )
        }

        @Serializable
        data class RestrictedCapabilities(
            @SerialName("type") val type: List<String>,
            @SerialName("items") val items: Items,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("minLength") val minLength: Int,
                @SerialName("maxLength") val maxLength: Int
            )
        }

        @Serializable
        data class Market(
            @SerialName("type") val type: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class MarketArray(
            @SerialName("type") val type: List<String>,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("items") val items: Items,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class Markets(
            @SerialName("description") val description: String,
            @SerialName("type") val type: List<String>,
            @SerialName("oneOf") val oneOf: List<OneOf>
        ) {
            @Serializable
            data class OneOf(
                @SerialName("properties") val properties: Properties,
                @SerialName("required") val required: List<String>
            ) {
                @Serializable
                data class Properties(
                    @SerialName("AllowedMarkets") val allowedMarkets: AllowedMarkets? = null,
                    @SerialName("ExcludedMarkets") val excludedMarkets: ExcludedMarkets? = null
                ) {
                    @Serializable
                    data class AllowedMarkets(
                        @SerialName("\$ref") val ref: String
                    )

                    @Serializable
                    data class ExcludedMarkets(
                        @SerialName("\$ref") val ref: String
                    )
                }
            }
        }

        @Serializable
        data class InstallerAbortsTerminal(
            @SerialName("type") val type: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ReleaseDate(
            @SerialName("type") val type: List<String>,
            @SerialName("format") val format: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class InstallLocationRequired(
            @SerialName("type") val type: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class RequireExplicitUpgrade(
            @SerialName("type") val type: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class DisplayInstallWarnings(
            @SerialName("type") val type: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class UnsupportedOSArchitectures(
            @SerialName("type") val type: List<String>,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("items") val items: Items,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("title") val title: String,
                @SerialName("enum") val enum: List<String>
            )
        }

        @Serializable
        data class UnsupportedArguments(
            @SerialName("type") val type: List<String>,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("items") val items: Items,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("type") val type: String,
                @SerialName("title") val title: String,
                @SerialName("enum") val enum: List<String>
            )
        }

        @Serializable
        data class AppsAndFeaturesEntry(
            @SerialName("type") val type: String,
            @SerialName("properties") val properties: Properties,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Properties(
                @SerialName("DisplayName") val displayName: DisplayName,
                @SerialName("Publisher") val publisher: Publisher,
                @SerialName("DisplayVersion") val displayVersion: DisplayVersion,
                @SerialName("ProductCode") val productCode: ProductCode,
                @SerialName("UpgradeCode") val upgradeCode: UpgradeCode,
                @SerialName("InstallerType") val installerType: InstallerType
            ) {
                @Serializable
                data class DisplayName(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Publisher(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class DisplayVersion(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class ProductCode(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class UpgradeCode(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerType(
                    @SerialName("\$ref") val ref: String
                )
            }
        }

        @Serializable
        data class AppsAndFeaturesEntries(
            @SerialName("type") val type: List<String>,
            @SerialName("uniqueItems") val uniqueItems: Boolean,
            @SerialName("maxItems") val maxItems: Int,
            @SerialName("items") val items: Items,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class ElevationRequirement(
            @SerialName("type") val type: List<String>,
            @SerialName("enum") val enum: List<String>,
            @SerialName("description") val description: String
        )

        @Serializable
        data class InstallationMetadata(
            @SerialName("type") val type: String,
            @SerialName("title") val title: String,
            @SerialName("properties") val properties: Properties,
            @SerialName("description") val description: String
        ) {
            @Serializable
            data class Properties(
                @SerialName("DefaultInstallLocation") val defaultInstallLocation: DefaultInstallLocation,
                @SerialName("Files") val files: Files
            ) {
                @Serializable
                data class DefaultInstallLocation(
                    @SerialName("type") val type: List<String>,
                    @SerialName("minLength") val minLength: Int,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class Files(
                    @SerialName("type") val type: List<String>,
                    @SerialName("uniqueItems") val uniqueItems: Boolean,
                    @SerialName("maxItems") val maxItems: Int,
                    @SerialName("items") val items: Items,
                    @SerialName("description") val description: String
                ) {
                    @Serializable
                    data class Items(
                        @SerialName("type") val type: String,
                        @SerialName("title") val title: String,
                        @SerialName("properties") val properties: Properties,
                        @SerialName("required") val required: List<String>,
                        @SerialName("description") val description: String
                    ) {
                        @Serializable
                        data class Properties(
                            @SerialName("RelativeFilePath") val relativeFilePath: RelativeFilePath,
                            @SerialName("FileSha256") val fileSha256: FileSha256,
                            @SerialName("FileType") val fileType: FileType,
                            @SerialName("InvocationParameter") val invocationParameter: InvocationParameter,
                            @SerialName("DisplayName") val displayName: DisplayName
                        ) {
                            @Serializable
                            data class RelativeFilePath(
                                @SerialName("type") val type: String,
                                @SerialName("minLength") val minLength: Int,
                                @SerialName("maxLength") val maxLength: Int,
                                @SerialName("description") val description: String
                            )

                            @Serializable
                            data class FileSha256(
                                @SerialName("type") val type: List<String>,
                                @SerialName("pattern") val pattern: String,
                                @SerialName("description") val description: String
                            )

                            @Serializable
                            data class FileType(
                                @SerialName("type") val type: List<String>,
                                @SerialName("enum") val enum: List<String>,
                                @SerialName("description") val description: String
                            )

                            @Serializable
                            data class InvocationParameter(
                                @SerialName("type") val type: List<String>,
                                @SerialName("minLength") val minLength: Int,
                                @SerialName("maxLength") val maxLength: Int,
                                @SerialName("description") val description: String
                            )

                            @Serializable
                            data class DisplayName(
                                @SerialName("type") val type: List<String>,
                                @SerialName("minLength") val minLength: Int,
                                @SerialName("maxLength") val maxLength: Int,
                                @SerialName("description") val description: String
                            )
                        }
                    }
                }
            }
        }

        @Serializable
        data class Installer(
            @SerialName("type") val type: String,
            @SerialName("properties") val properties: Properties,
            @SerialName("required") val required: List<String>
        ) {
            @Serializable
            data class Properties(
                @SerialName("InstallerLocale") val installerLocale: InstallerLocale,
                @SerialName("Platform") val platform: Platform,
                @SerialName("MinimumOSVersion") val minimumOSVersion: MinimumOSVersion,
                @SerialName("Architecture") val architecture: Architecture,
                @SerialName("InstallerType") val installerType: InstallerType,
                @SerialName("NestedInstallerType") val nestedInstallerType: NestedInstallerType,
                @SerialName("NestedInstallerFiles") val nestedInstallerFiles: NestedInstallerFiles,
                @SerialName("Scope") val scope: Scope,
                @SerialName("InstallerUrl") val installerUrl: InstallerUrl,
                @SerialName("InstallerSha256") val installerSha256: InstallerSha256,
                @SerialName("SignatureSha256") val signatureSha256: SignatureSha256,
                @SerialName("InstallModes") val installModes: InstallModes,
                @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches,
                @SerialName("InstallerSuccessCodes") val installerSuccessCodes: InstallerSuccessCodes,
                @SerialName("ExpectedReturnCodes") val expectedReturnCodes: ExpectedReturnCodes,
                @SerialName("UpgradeBehavior") val upgradeBehavior: UpgradeBehavior,
                @SerialName("Commands") val commands: Commands,
                @SerialName("Protocols") val protocols: Protocols,
                @SerialName("FileExtensions") val fileExtensions: FileExtensions,
                @SerialName("Dependencies") val dependencies: Dependencies,
                @SerialName("PackageFamilyName") val packageFamilyName: PackageFamilyName,
                @SerialName("ProductCode") val productCode: ProductCode,
                @SerialName("Capabilities") val capabilities: Capabilities,
                @SerialName("RestrictedCapabilities") val restrictedCapabilities: RestrictedCapabilities,
                @SerialName("Markets") val markets: Markets,
                @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: InstallerAbortsTerminal,
                @SerialName("ReleaseDate") val releaseDate: ReleaseDate,
                @SerialName("InstallLocationRequired") val installLocationRequired: InstallLocationRequired,
                @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: RequireExplicitUpgrade,
                @SerialName("DisplayInstallWarnings") val displayInstallWarnings: DisplayInstallWarnings,
                @SerialName("UnsupportedOSArchitectures") val unsupportedOSArchitectures: UnsupportedOSArchitectures,
                @SerialName("UnsupportedArguments") val unsupportedArguments: UnsupportedArguments,
                @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: AppsAndFeaturesEntries,
                @SerialName("ElevationRequirement") val elevationRequirement: ElevationRequirement,
                @SerialName("InstallationMetadata") val installationMetadata: InstallationMetadata
            ) {
                @Serializable
                data class InstallerLocale(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Platform(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class MinimumOSVersion(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Architecture(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerType(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class NestedInstallerType(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class NestedInstallerFiles(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Scope(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerUrl(
                    @SerialName("type") val type: String,
                    @SerialName("pattern") val pattern: String,
                    @SerialName("maxLength") val maxLength: Int,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class InstallerSha256(
                    @SerialName("type") val type: String,
                    @SerialName("pattern") val pattern: String,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class SignatureSha256(
                    @SerialName("type") val type: List<String>,
                    @SerialName("pattern") val pattern: String,
                    @SerialName("description") val description: String
                )

                @Serializable
                data class InstallModes(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerSwitches(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerSuccessCodes(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class ExpectedReturnCodes(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class UpgradeBehavior(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Commands(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Protocols(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class FileExtensions(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Dependencies(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class PackageFamilyName(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class ProductCode(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Capabilities(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class RestrictedCapabilities(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class Markets(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallerAbortsTerminal(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class ReleaseDate(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallLocationRequired(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class RequireExplicitUpgrade(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class DisplayInstallWarnings(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class UnsupportedOSArchitectures(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class UnsupportedArguments(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class AppsAndFeaturesEntries(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class ElevationRequirement(
                    @SerialName("\$ref") val ref: String
                )

                @Serializable
                data class InstallationMetadata(
                    @SerialName("\$ref") val ref: String
                )
            }
        }
    }

    @Serializable
    data class Properties(
        @SerialName("PackageIdentifier") val packageIdentifier: PackageIdentifier,
        @SerialName("PackageVersion") val packageVersion: PackageVersion,
        @SerialName("Channel") val channel: Channel,
        @SerialName("InstallerLocale") val installerLocale: InstallerLocale,
        @SerialName("Platform") val platform: Platform,
        @SerialName("MinimumOSVersion") val minimumOSVersion: MinimumOSVersion,
        @SerialName("InstallerType") val installerType: InstallerType,
        @SerialName("NestedInstallerType") val nestedInstallerType: NestedInstallerType,
        @SerialName("NestedInstallerFiles") val nestedInstallerFiles: NestedInstallerFiles,
        @SerialName("Scope") val scope: Scope,
        @SerialName("InstallModes") val installModes: InstallModes,
        @SerialName("InstallerSwitches") val installerSwitches: InstallerSwitches,
        @SerialName("InstallerSuccessCodes") val installerSuccessCodes: InstallerSuccessCodes,
        @SerialName("ExpectedReturnCodes") val expectedReturnCodes: ExpectedReturnCodes,
        @SerialName("UpgradeBehavior") val upgradeBehavior: UpgradeBehavior,
        @SerialName("Commands") val commands: Commands,
        @SerialName("Protocols") val protocols: Protocols,
        @SerialName("FileExtensions") val fileExtensions: FileExtensions,
        @SerialName("Dependencies") val dependencies: Dependencies,
        @SerialName("PackageFamilyName") val packageFamilyName: PackageFamilyName,
        @SerialName("ProductCode") val productCode: ProductCode,
        @SerialName("Capabilities") val capabilities: Capabilities,
        @SerialName("RestrictedCapabilities") val restrictedCapabilities: RestrictedCapabilities,
        @SerialName("Markets") val markets: Markets,
        @SerialName("InstallerAbortsTerminal") val installerAbortsTerminal: InstallerAbortsTerminal,
        @SerialName("ReleaseDate") val releaseDate: ReleaseDate,
        @SerialName("InstallLocationRequired") val installLocationRequired: InstallLocationRequired,
        @SerialName("RequireExplicitUpgrade") val requireExplicitUpgrade: RequireExplicitUpgrade,
        @SerialName("DisplayInstallWarnings") val displayInstallWarnings: DisplayInstallWarnings,
        @SerialName("UnsupportedOSArchitectures") val unsupportedOSArchitectures: UnsupportedOSArchitectures,
        @SerialName("UnsupportedArguments") val unsupportedArguments: UnsupportedArguments,
        @SerialName("AppsAndFeaturesEntries") val appsAndFeaturesEntries: AppsAndFeaturesEntries,
        @SerialName("ElevationRequirement") val elevationRequirement: ElevationRequirement,
        @SerialName("InstallationMetadata") val installationMetadata: InstallationMetadata,
        @SerialName("Installers") val installers: Installers,
        @SerialName("ManifestType") val manifestType: ManifestType,
        @SerialName("ManifestVersion") val manifestVersion: ManifestVersion
    ) {
        @Serializable
        data class PackageIdentifier(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class PackageVersion(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Channel(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallerLocale(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Platform(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class MinimumOSVersion(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallerType(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class NestedInstallerType(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class NestedInstallerFiles(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Scope(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallModes(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallerSwitches(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallerSuccessCodes(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class ExpectedReturnCodes(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class UpgradeBehavior(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Commands(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Protocols(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class FileExtensions(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Dependencies(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class PackageFamilyName(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class ProductCode(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Capabilities(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class RestrictedCapabilities(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Markets(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallerAbortsTerminal(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class ReleaseDate(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallLocationRequired(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class RequireExplicitUpgrade(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class DisplayInstallWarnings(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class UnsupportedOSArchitectures(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class UnsupportedArguments(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class AppsAndFeaturesEntries(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class ElevationRequirement(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class InstallationMetadata(
            @SerialName("\$ref") val ref: String
        )

        @Serializable
        data class Installers(
            @SerialName("type") val type: String,
            @SerialName("items") val items: Items,
            @SerialName("minItems") val minItems: Int,
            @SerialName("maxItems") val maxItems: Int
        ) {
            @Serializable
            data class Items(
                @SerialName("\$ref") val ref: String
            )
        }

        @Serializable
        data class ManifestType(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("const") val const: String,
            @SerialName("description") val description: String
        )

        @Serializable
        data class ManifestVersion(
            @SerialName("type") val type: String,
            @SerialName("default") val default: String,
            @SerialName("pattern") val pattern: String,
            @SerialName("description") val description: String
        )
    }
}
