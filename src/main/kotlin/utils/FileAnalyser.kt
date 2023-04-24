package utils

import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import extensions.extension
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

class FileAnalyser(private val file: Path, private val fileSystem: FileSystem) {
    private val msi = if (file.extension == InstallerType.MSI.toString()) Msi(file, fileSystem) else null
    private val msix = when (file.extension) {
        InstallerType.MSIX.toString(), InstallerType.APPX.toString() -> Msix(file.toFile())
        else -> null
    }
    private val msixBundle = when (file.extension) {
        MsixBundle.msixBundleConst, MsixBundle.appxBundleConst -> MsixBundle(file.toFile())
        else -> null
    }

    val installerType: InstallerType?
        get() = when (file.extension) {
            InstallerType.MSI.toString() -> if (msi?.isWix == true) InstallerType.WIX else InstallerType.MSI
            InstallerType.ZIP.toString() -> InstallerType.ZIP
            InstallerType.APPX.toString() -> InstallerType.APPX
            InstallerType.MSIX.toString() -> InstallerType.MSIX
            else -> fileSystem.openReadOnly(file).use {
                when {
                    it.isNullsoft -> InstallerType.NULLSOFT
                    it.isInno -> InstallerType.INNO
                    it.isBurn -> InstallerType.BURN
                    else -> null
                }
            }
        }

    val architecture: Architecture
        get() = when (file.extension) {
            InstallerType.MSI.toString() -> msi?.architecture
            InstallerType.MSIX.toString() -> {
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

    val signatureSha256: String? get() = msix?.signatureSha256 ?: msixBundle?.signatureSha256

    val productCode: String? get() = msi?.productCode

    val upgradeBehaviour: UpgradeBehavior?
        get() {
            val validExtensions = listOf(
                InstallerType.APPX.toString(),
                InstallerType.MSIX.toString(),
                MsixBundle.appxBundleConst,
                MsixBundle.msixBundleConst
            )
            return if (file.extension in validExtensions) UpgradeBehavior.Install else null
        }

    val scope: Scope?
        get() = when {
            msi?.allUsers != null -> msi.allUsers?.toInstallerScope()
            msix != null || msixBundle != null -> Scope.User
            else -> null
        }

    val minVersion: String? get() = msix?.minVersion ?: msixBundle?.packages?.first()?.minVersion

    /**
     * Returns `true` if the [FileHandle] has been made with the Nullsoft Scriptable Install System.
     *
     * This works by comparing the first 224 bytes of the file to [nullsoftBytes].
     *
     * @return `true` if the [FileHandle] has been made with NSIS, `false` otherwise.
     */
    private val FileHandle.isNullsoft: Boolean
        get() = Buffer().also { read(0L, it, nullsoftBytes.size.toLong()) }.rangeEquals(0L, nullsoftBytes)

    /**
     * Returns `true` if the [FileHandle] has been made with Inno Setup.
     *
     * This works by comparing the first 264 bytes of the file to [innoBytes].
     *
     * @return `true` if the [FileHandle] has been made with Inno Setup, `false` otherwise.
     */
    private val FileHandle.isInno: Boolean
        get() = Buffer().also { read(0L, it, innoBytes.size.toLong()) }.rangeEquals(0L, innoBytes)

    /**
     * Returns `true` if the [FileHandle] has been made with WiX's burn installer type.
     *
     * This works by searching for the `.wixburn` header in a specific section of the binary.
     *
     * See [GetWixburnSectionInfo](https://github.com/AnalogJ/Wix3.6Toolset/blob/master/RC0-source/wix36-sources/src/wix/BurnCommon.cs#L252) in WiX Toolset v3.
     * @return `true` if the [FileHandle] has been made with WiX's burn installer type, `false` otherwise.
     */
    private val FileHandle.isBurn: Boolean
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
