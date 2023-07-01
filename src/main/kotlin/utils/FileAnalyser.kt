package utils

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.platform.win32.Kernel32Util
import okio.Buffer
import okio.ByteString
import okio.ByteString.Companion.encodeUtf8
import okio.FileHandle
import okio.FileSystem
import okio.Path
import okio.buffer
import schemas.manifest.InstallerManifest.Installer.Architecture
import schemas.manifest.InstallerManifest.InstallerType
import schemas.manifest.InstallerManifest.Scope
import schemas.manifest.InstallerManifest.UpgradeBehavior
import utils.jna.Kernel32
import utils.msi.Msi
import utils.msix.Msix
import utils.msix.MsixBundle

class FileAnalyser(private val file: Path, private val fileSystem: FileSystem = FileSystem.SYSTEM) {
    private val resourceNamesMap: Map<String, List<String>>? by lazy {
        if (Platform.isWindows()) {
            runCatching { Kernel32Util.getResourceNames(file.toString()) }.getOrNull()
        } else {
            null
        }
    }

    val msix = when {
        file.extension.equals(InstallerType.MSIX.name, ignoreCase = true) ||
            file.extension.equals(InstallerType.APPX.name, ignoreCase = true) -> Msix(file)
        else -> null
    }

    val msi = if (file.extension.lowercase() == InstallerType.MSI.toString()) Msi(file) else null

    val msixBundle = when {
        file.extension.equals(MsixBundle.msixBundleConst, ignoreCase = true) ||
            file.extension.equals(MsixBundle.appxBundleConst, ignoreCase = true) -> MsixBundle(file)
        else -> null
    }

    private val manifest: String? by lazy {
        if (Platform.isWindows()) {
            if (resourceNamesMap?.get(Kernel32.RT_MANIFEST)?.contains(Kernel32.MANIFEST_RESOURCE) == true) {
                val manifestBytes = Kernel32Util.getResource(file.toString(), Kernel32.RT_MANIFEST, Kernel32.MANIFEST_RESOURCE)
                Native.toString(manifestBytes)
            } else {
                null
            }
        } else {
            val startTag = "<assembly"
            val endTag = "</assembly>"
            var record = false
            buildString {
                fileSystem.source(file).buffer().use { buffer ->
                    while (!buffer.exhausted()) {
                        val line = buffer.readUtf8Line() ?: break
                        if (line.contains(startTag)) {
                            val startIndex = line.indexOf(startTag)
                            appendLine(line.substring(startIndex))
                            record = true
                        } else if (record) {
                            if (line.contains(endTag)) {
                                val endIndex = line.indexOf(endTag) + endTag.length
                                append(line.substring(0, endIndex))
                                record = false
                            } else {
                                appendLine(line)
                            }
                        }
                    }
                }
            }.ifBlank { null }
        }
    }

    val installerType: InstallerType? by lazy {
        when {
            file.extension.equals(InstallerType.MSI.name, ignoreCase = true) -> {
                if (msi?.isWix == true) InstallerType.WIX else InstallerType.MSI
            }
            file.extension.equals(InstallerType.ZIP.name, ignoreCase = true) -> InstallerType.ZIP
            file.extension.equals(InstallerType.APPX.name, ignoreCase = true) -> InstallerType.APPX
            file.extension.equals(InstallerType.MSIX.name, ignoreCase = true) -> InstallerType.MSIX
            else -> fileSystem.openReadOnly(file).use {
                when {
                    it.isNullsoft -> InstallerType.NULLSOFT
                    it.isInno -> InstallerType.INNO
                    resourceNamesMap?.get(Kernel32.RT_RCDATA)?.contains(Kernel32.MSI_RESOURCE) == true ||
                        it.hasBurnHeader -> InstallerType.BURN
                    else -> null
                }
            }
        }
    }

    val productCode: String? by lazy {
        if (file.extension.equals(InstallerType.MSI.name, ignoreCase = true)) {
            msi?.productCode
        } else if (Platform.isWindows() && file.extension.equals(InstallerType.EXE.name, ignoreCase = true)) {
            if (resourceNamesMap?.get(Kernel32.RT_RCDATA)?.contains(Kernel32.PRODUCT_CODE_RESOURCE) == true) {
                val productCodeBytes = Kernel32Util.getResource(file.toString(), Kernel32.RT_RCDATA, Kernel32.PRODUCT_CODE_RESOURCE)
                Native.toString(productCodeBytes)
            } else {
                null
            }
        } else {
            null
        }
    }

    val architecture: Architecture by lazy {
        when {
            file.extension.equals(InstallerType.MSI.name, ignoreCase = true) -> msi?.architecture
            file.extension.equals(InstallerType.MSIX.name, ignoreCase = true) -> {
                msix?.processorArchitecture ?: msixBundle?.packages?.first()?.processorArchitecture
            }
            else -> when (peArchitectureValue) {
                "8664" -> Architecture.X64
                "14c" -> Architecture.X86
                "aa64" -> Architecture.ARM64
                "1c0", "1c4" -> Architecture.ARM
                else -> null
            }
        } ?: Architecture.X64
    }

    val upgradeBehaviour: UpgradeBehavior?
        get() {
            val validExtensions = listOf(
                InstallerType.APPX.toString(),
                InstallerType.MSIX.toString(),
                MsixBundle.appxBundleConst,
                MsixBundle.msixBundleConst
            )
            return if (file.extension.lowercase() in validExtensions) UpgradeBehavior.Install else null
        }

    val scope: Scope?
        get() = when {
            msi?.allUsers != null -> msi.allUsers?.toInstallerScope()
            msix != null || msixBundle != null -> Scope.User
            else -> null
        }

    val publisherDisplayName: String? = msi?.manufacturer ?: msix?.publisherDisplayName

    /**
     * Returns `true` if the [FileHandle] has been made with the Nullsoft Scriptable Install System.
     *
     * This works by comparing the first 224 bytes of the file to [nullsoftBytes].
     *
     * @return `true` if the [FileHandle] has been made with NSIS, `false` otherwise.
     */
    private val FileHandle.isNullsoft: Boolean
        get() = Buffer().use {
            read(0L, it, nullsoftBytes.size.toLong())
            it.rangeEquals(0L, nullsoftBytes)
        }

    /**
     * Returns `true` if the [FileHandle] has been made with Inno Setup.
     *
     * This works by comparing the first 264 bytes of the file to [innoBytes].
     *
     * @return `true` if the [FileHandle] has been made with Inno Setup, `false` otherwise.
     */
    private val FileHandle.isInno: Boolean
        get() = Buffer().use {
            read(0L, it, innoBytes.size.toLong())
            it.rangeEquals(0L, innoBytes)
        }

    /**
     * Returns `true` if the [FileHandle] has been made with WiX's burn installer type.
     *
     * This works by searching for the `.wixburn` header in a specific section of the binary.
     *
     * See [GetWixburnSectionInfo](https://github.com/AnalogJ/Wix3.6Toolset/blob/master/RC0-source/wix36-sources/src/wix/BurnCommon.cs#L252) in WiX Toolset v3.
     * @return `true` if the [FileHandle] has been made with WiX's burn installer type, `false` otherwise.
     */
    private val FileHandle.hasBurnHeader: Boolean
        get() {
            source().buffer().use { bufferedSource ->
                val sink = Buffer()
                var offset = 0L
                val wixBurnHeaderBytes = wixBurnHeader.encodeUtf8()
                repeat(UShort.MAX_VALUE.toInt()) {
                    bufferedSource.read(sink, burnBufferSize)
                    if (sink.rangeEquals(offset, wixBurnHeaderBytes)) return true
                    offset += burnBufferSize
                }
                return false
            }
        }

    /**
     * Returns the hexadecimal string representation of the machine value
     * stored in the PE header of the given file.
     *
     * This function reads the PE header of the file and extracts the machine
     * value, which represents the target architecture of the binary. The machine
     * value is a 2-byte little-endian integer stored at a specific offset in the
     * PE header.
     *
     * @return The hexadecimal string representation of the machine value.
     */
    val peArchitectureValue: String?
        get() {
            return runCatching {
                fileSystem.source(file).buffer().use { source ->
                    // Skip DOS header
                    source.skip(peHeaderLocation)
                    val peOffset = source.readIntLe()

                    // Skip PE signature
                    source.skip(peOffset - peHeaderLocation)

                    // Read machine value from PE header
                    val machine = source.readShortLe() // Machine is stored as a 2-byte little-endian value

                    machine.toInt().and(UShort.MAX_VALUE.toInt()).toString(hexBase16)
                }
            }.getOrNull()
        }

    companion object {
        const val burnBufferSize: Long = 8
        const val wixBurnHeader = ".wixburn"
        const val peHeaderLocation: Long = 0x3C
        private const val hexBase16 = 16

        /**
         * The first 224 bytes of an exe made with NSIS are the same
         */
        val nullsoftBytes = ByteString.of(
            77, 90, -112, 0, 3, 0, 0, 0, 4, 0, 0, 0, -1, -1, 0, 0, -72, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -40, 0, 0, 0, 14, 31, -70,
            14, 0, -76, 9, -51, 33, -72, 1, 76, -51, 33, 84, 104, 105, 115, 32, 112, 114, 111, 103, 114, 97, 109, 32,
            99, 97, 110, 110, 111, 116, 32, 98, 101, 32, 114, 117, 110, 32, 105, 110, 32, 68, 79, 83, 32, 109, 111, 100,
            101, 46, 13, 13, 10, 36, 0, 0, 0, 0, 0, 0, 0, -83, 49, 8, -127, -23, 80, 102, -46, -23, 80, 102, -46, -23,
            80, 102, -46, 42, 95, 57, -46, -21, 80, 102, -46, -23, 80, 103, -46, 76, 80, 102, -46, 42, 95, 59, -46, -26,
            80, 102, -46, -67, 115, 86, -46, -29, 80, 102, -46, 46, 86, 96, -46, -24, 80, 102, -46, 82, 105, 99, 104,
            -23, 80, 102, -46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 69, 0, 0, 76,
            1, 5, 0
        )

        /**
         * The first 264 bytes of an exe made with Inno Setup are the same
         */
        val innoBytes = ByteString.of(
            77, 90, 80, 0, 2, 0, 0, 0, 4, 0, 15, 0, -1, -1, 0, 0, -72, 0, 0, 0, 0, 0, 0, 0, 64, 0, 26, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, -70, 16, 0, 14,
            31, -76, 9, -51, 33, -72, 1, 76, -51, 33, -112, -112, 84, 104, 105, 115, 32, 112, 114, 111, 103, 114, 97,
            109, 32, 109, 117, 115, 116, 32, 98, 101, 32, 114, 117, 110, 32, 117, 110, 100, 101, 114, 32, 87, 105, 110,
            51, 50, 13, 10, 36, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            80, 69, 0, 0, 76, 1, 10, 0
        )
    }
}
