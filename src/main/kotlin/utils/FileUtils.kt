package utils

import schemas.manifest.InstallerManifest
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder

object FileUtils {
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

    private const val peHeader: Long = 0x3C
    private const val readOnly = "r"
}
