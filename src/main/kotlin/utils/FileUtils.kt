package utils

import detection.files.msi.Msi
import detection.files.msix.Msix
import detection.files.msix.MsixBundle
import schemas.manifest.InstallerManifest.Installer.Architecture
import schemas.manifest.InstallerManifest.Installer.InstallerType
import schemas.manifest.InstallerManifest.Installer.UpgradeBehavior
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder

class FileUtils(private val file: File) {
    private val msi = if (file.extension == InstallerType.MSI.toString()) Msi(file) else null
    private val msix = when (file.extension) {
        InstallerType.MSIX.toString(), InstallerType.APPX.toString() -> Msix(file)
        else -> null
    }
    private val msixBundle = when (file.extension) {
        MsixBundle.msixBundleConst, MsixBundle.appxBundleConst -> MsixBundle(file)
        else -> null
    }

    fun getInstallerType(): InstallerType? {
        return when (file.extension) {
            InstallerType.MSI.toString() -> if (msi?.isWix == true) InstallerType.WIX else InstallerType.MSI
            InstallerType.ZIP.toString() -> InstallerType.ZIP
            InstallerType.APPX.toString() -> InstallerType.APPX
            InstallerType.MSIX.toString() -> InstallerType.MSIX
            else -> {
                RandomAccessFile(file, readOnly).use {
                    return when {
                        it.isNullsoft() -> InstallerType.NULLSOFT
                        it.isInno() -> InstallerType.INNO
                        it.isBurn() -> InstallerType.BURN
                        else -> null
                    }
                }
            }
        }
    }

    fun getArchitecture(): Architecture? {
        return when (file.extension) {
            InstallerType.MSI.toString() -> msi?.architecture
            InstallerType.MSIX.toString() -> {
                msix?.processorArchitecture ?: msixBundle?.packages?.first()?.processorArchitecture
            }
            else -> when (getPEArchitectureValue()) {
                "8664" -> Architecture.X64
                "14c" -> Architecture.X86
                "aa64" -> Architecture.ARM64
                "1c0", "1c4" -> Architecture.ARM
                else -> null
            }
        }
    }

    fun getSignatureSha256(): String? = msix?.signatureSha256 ?: msixBundle?.signatureSha256

    fun getProductCode(): String? = msi?.productCode

    fun getUpgradeBehaviour(): UpgradeBehavior? {
        val validExtensions = listOf(
            InstallerType.APPX.toString(),
            InstallerType.MSIX.toString(),
            MsixBundle.appxBundleConst,
            MsixBundle.msixBundleConst
        )
        return if (file.extension in validExtensions) UpgradeBehavior.Install else null
    }

    /**
     * Returns `true` if the [RandomAccessFile] has been made with the Nullsoft Scriptable Install System.
     *
     * This works by comparing the first 224 bytes of the file to [nullsoftBytes].
     *
     * @return `true` if the [RandomAccessFile] has been made with NSIS, `false` otherwise.
     */
    private fun RandomAccessFile.isNullsoft(): Boolean {
        val magicBytes = ByteArray(nullsoftBytes.size)
        seek(0)
        read(magicBytes)
        return magicBytes.contentEquals(nullsoftBytes)
    }

    /**
     * Returns `true` if the [RandomAccessFile] has been made with Inno Setup.
     *
     * This works by comparing the first 264 bytes of the file to [innoBytes].
     *
     * @return `true` if the [RandomAccessFile] has been made with Inno Setup, `false` otherwise.
     */
    private fun RandomAccessFile.isInno(): Boolean {
        val magicBytes = ByteArray(innoBytes.size)
        seek(0)
        read(magicBytes)
        return magicBytes.contentEquals(innoBytes)
    }

    /**
     * Returns `true` if the [RandomAccessFile] has been made with WiX's burn installer type.
     *
     * This works by searching for the `.wixburn` header in a specific section of the binary.
     *
     * See [GetWixburnSectionInfo](https://github.com/AnalogJ/Wix3.6Toolset/blob/master/RC0-source/wix36-sources/src/wix/BurnCommon.cs#L252) in WiX Toolset v3.
     * @return `true` if the [RandomAccessFile] has been made with WiX's burn installer type, `false` otherwise.
     */
    private fun RandomAccessFile.isBurn(): Boolean {
        val bytes = ByteArray(8)
        seek(0)
        skipBytes(UInt.MAX_VALUE.toInt())
        repeat(UShort.MAX_VALUE.toInt()) {
            read(bytes)
            if (bytes.contentEquals(wixBurnHeader.toByteArray())) {
                return true
            }
        }
        return false
    }

    private fun getPEArchitectureValue(): String {
        RandomAccessFile(file, readOnly).use {
            it.seek(peHeaderLocation)
            val peSignatureBuffer = ByteArray(Int.SIZE_BYTES)
            it.read(peSignatureBuffer)
            val offset: Int = ByteBuffer.wrap(peSignatureBuffer).order(ByteOrder.LITTLE_ENDIAN).int
            it.seek((offset + Int.SIZE_BYTES).toLong())
            val machineBuffer = ByteArray(Short.SIZE_BYTES)
            it.read(machineBuffer)
            val machine: Short = ByteBuffer.wrap(machineBuffer).order(ByteOrder.LITTLE_ENDIAN).short
            return Integer.toHexString(machine.toInt() and UShort.MAX_VALUE.toInt())
        }
    }

    /**
     * The first 224 bytes of an exe made with NSIS are the same
     */
    private val nullsoftBytes = byteArrayOf(
        77, 90, -112, 0, 3, 0, 0, 0, 4, 0, 0, 0, -1, -1, 0, 0, -72, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -40, 0, 0, 0, 14, 31, -70, 14, 0,
        -76, 9, -51, 33, -72, 1, 76, -51, 33, 84, 104, 105, 115, 32, 112, 114, 111, 103, 114, 97, 109, 32, 99, 97, 110,
        110, 111, 116, 32, 98, 101, 32, 114, 117, 110, 32, 105, 110, 32, 68, 79, 83, 32, 109, 111, 100, 101, 46, 13, 13,
        10, 36, 0, 0, 0, 0, 0, 0, 0, -83, 49, 8, -127, -23, 80, 102, -46, -23, 80, 102, -46, -23, 80, 102, -46, 42, 95,
        57, -46, -21, 80, 102, -46, -23, 80, 103, -46, 76, 80, 102, -46, 42, 95, 59, -46, -26, 80, 102, -46, -67, 115,
        86, -46, -29, 80, 102, -46, 46, 86, 96, -46, -24, 80, 102, -46, 82, 105, 99, 104, -23, 80, 102, -46, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 69, 0, 0, 76, 1, 5, 0
    )

    /**
     * The first 264 bytes of an exe made with Inno Setup are the same
     */
    private val innoBytes = byteArrayOf(
        77, 90, 80, 0, 2, 0, 0, 0, 4, 0, 15, 0, -1, -1, 0, 0, -72, 0, 0, 0, 0, 0, 0, 0, 64, 0, 26, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, -70, 16, 0, 14, 31,
        -76, 9, -51, 33, -72, 1, 76, -51, 33, -112, -112, 84, 104, 105, 115, 32, 112, 114, 111, 103, 114, 97, 109, 32,
        109, 117, 115, 116, 32, 98, 101, 32, 114, 117, 110, 32, 117, 110, 100, 101, 114, 32, 87, 105, 110, 51, 50, 13,
        10, 36, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 69, 0, 0, 76, 1, 10,
        0
    )

    companion object {
        private const val wixBurnHeader: String = ".wixburn"
        private const val peHeaderLocation: Long = 0x3C
        private const val readOnly = "r"
    }
}
