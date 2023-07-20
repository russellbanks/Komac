package utils.msi

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.Pointer
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import schemas.manifest.InstallerManifest
import utils.jna.GObject
import utils.jna.LibMsi
import utils.jna.WinMsi

class MsiArch(private val database: Pointer) {

    val architecture: InstallerManifest.Installer.Architecture? by lazy {
        when {
            Platform.isWindows() -> getWindowsArchitecture()
            Platform.isLinux() -> getLinuxArchitecture()
            else -> null
        }?.split(';')?.first()?.toArchitecture()
    }

    private fun getWindowsArchitecture(): String? {
        val winMsi = WinMsi.INSTANCE
        val phSummaryInfo = PointerByReference()
        var result = winMsi.MsiGetSummaryInformation(database, null, UI_UPDATE_COUNT, phSummaryInfo)
        if (result != WinMsi.Errors.ERROR_SUCCESS) return null

        val pcchBuf = IntByReference()
        var szBuf: CharArray
        do {
            val bufferSize = if (pcchBuf.value <= 0) 16 else pcchBuf.value + 1
            szBuf = CharArray(bufferSize)
            pcchBuf.value = bufferSize
            result = winMsi.MsiSummaryInfoGetProperty(
                hSummaryInfo = phSummaryInfo.value,
                uiProperty = PROPERTY_TEMPLATE,
                puiDataType = IntByReference(),
                piValue = IntByReference(),
                pftValue = FILETIME(),
                szValueBuf = szBuf,
                pcchValueBuf = pcchBuf
            )
        } while (result == WinMsi.Errors.ERROR_MORE_DATA)
        winMsi.MsiCloseHandle(phSummaryInfo.value)
        return if (result == WinMsi.Errors.ERROR_SUCCESS) Native.toString(szBuf) else null
    }

    private fun getLinuxArchitecture(): String {
        val libMsi = LibMsi.INSTANCE
        val gObject = GObject.INSTANCE

        val error = PointerByReference()
        val summaryInfo = libMsi.libmsiSummaryInfoNew(database, UI_UPDATE_COUNT, error)
        val template = libMsi.libmsiSummaryInfoGetString(summaryInfo, PROPERTY_TEMPLATE, error)

        gObject.gObjectUnref(summaryInfo)
        if (error.value != null) {
            gObject.gClearError(error.pointer)
        }
        return template
    }

    companion object {
        private const val PROPERTY_TEMPLATE = 7
        private const val UI_UPDATE_COUNT = 0

        fun String.toArchitecture(): InstallerManifest.Installer.Architecture? = when (this) {
            "x64", "Intel64", "AMD64" -> InstallerManifest.Installer.Architecture.X64
            "Intel" -> InstallerManifest.Installer.Architecture.X86
            else -> null
        }
    }
}
