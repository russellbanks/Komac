package utils

import schemas.manifest.InstallerManifest
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder

object ExeUtils {
    fun File.getInstallerType(): InstallerManifest.Installer.InstallerType? {
        RandomAccessFile(this, readOnly).use {
            return when {
                it.isNullsoft() -> InstallerManifest.Installer.InstallerType.NULLSOFT
                it.isInno() -> InstallerManifest.Installer.InstallerType.INNO
                it.isBurn() -> InstallerManifest.Installer.InstallerType.BURN
                else -> null
            }
        }
    }

    private fun RandomAccessFile.isNullsoft(): Boolean {
        val magicBytes = ByteArray(nullsoftBytes.size)
        seek(0)
        read(magicBytes)
        return magicBytes.contentEquals(nullsoftBytes)
    }

    private fun RandomAccessFile.isInno(): Boolean {
        val magicBytes = ByteArray(innoBytes.size)
        seek(0)
        read(magicBytes)
        return magicBytes.contentEquals(innoBytes)
    }

    private fun RandomAccessFile.isBurn(): Boolean {
        val bytes = ByteArray(8)
        seek(0)
        skipBytes(UInt.MAX_VALUE.toInt())
        for (index in 0 until UShort.MAX_VALUE.toInt()) {
            read(bytes)
            if (bytes.contentEquals(wixBurnHeader.toByteArray())) {
                return true
            }
        }
        return false
    }

    fun File.getArchitecture(): InstallerManifest.Installer.Architecture {
        return when (getMachineValue(this)) {
            "0x8664" -> InstallerManifest.Installer.Architecture.X64
            "0x14c" -> InstallerManifest.Installer.Architecture.X86
            "0xaa64" -> InstallerManifest.Installer.Architecture.ARM64
            "0x1c0", "0x1c4" -> InstallerManifest.Installer.Architecture.ARM
            else -> InstallerManifest.Installer.Architecture.NEUTRAL
        }
    }

    private fun getMachineValue(file: File): String {
        RandomAccessFile(file, readOnly).use {
            it.seek(peHeader)
            val peSignatureBuffer = ByteArray(Int.SIZE_BYTES)
            it.read(peSignatureBuffer)
            val offset: Int = ByteBuffer.wrap(peSignatureBuffer).order(ByteOrder.LITTLE_ENDIAN).int
            it.seek((offset + Int.SIZE_BYTES).toLong())
            val machineBuffer = ByteArray(Short.SIZE_BYTES)
            it.read(machineBuffer)
            val machine: Short = ByteBuffer.wrap(machineBuffer).order(ByteOrder.LITTLE_ENDIAN).short
            return "0x${Integer.toHexString(machine.toInt() and UShort.MAX_VALUE.toInt())}"
        }
    }

    /**
     * The first 224 bytes of a nullsoft exe are the same
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
     * The first 264 bytes of an inno exe are the same
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

    private const val wixBurnHeader: String = ".wixburn"
    private const val peHeader: Long = 0x3C
    private const val readOnly = "r"
}
