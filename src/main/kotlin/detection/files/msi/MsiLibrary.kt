package detection.files.msi

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.WString
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import schemas.manifest.InstallerManifest

@Suppress("FunctionName", "LongParameterList")
interface MsiLibrary : Library {
    fun MsiOpenDatabaseW(szDatabasePath: WString?, szPersist: WString?, phDatabase: PointerByReference?): Int
    fun MsiDatabaseOpenViewW(hDatabase: Pointer?, szQuery: WString?, phView: PointerByReference?): Int
    fun MsiViewExecute(hView: Pointer?, hRecord: Pointer?): Int
    fun MsiViewFetch(hView: Pointer?, phRecord: PointerByReference?): Int
    fun MsiRecordGetStringW(hRecord: Pointer?, iField: Int, szValueBuf: CharArray?, pcchValueBuf: IntByReference?): Int
    fun MsiCloseHandle(hAny: Pointer?): Int
    fun MsiGetSummaryInformationW(
        hDatabase: Pointer?,
        szDatabasePath: WString?,
        uiUpdateCount: Int,
        phSummaryInfo: PointerByReference?
    ): Int
    fun MsiSummaryInfoGetPropertyW(
        hSummaryInfo: Pointer,
        uiProperty: Int,
        puiDataType: IntByReference,
        piValue: IntByReference,
        pftValue: FILETIME,
        szValueBuf: CharArray,
        pcchValueBuf: IntByReference
    ): Int

    companion object {
        val INSTANCE = Native.load(InstallerManifest.InstallerType.MSI.toString(), MsiLibrary::class.java) as MsiLibrary
    }
}
