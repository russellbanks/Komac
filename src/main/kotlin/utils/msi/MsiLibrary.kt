package utils.msi

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.WString
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import com.sun.jna.win32.W32APIOptions

@Suppress("FunctionName", "LongParameterList")
interface MsiLibrary : Library {
    fun MsiOpenDatabase(szDatabasePath: WString?, szPersist: WString?, phDatabase: PointerByReference?): Int
    fun MsiDatabaseOpenView(hDatabase: Pointer?, szQuery: WString?, phView: PointerByReference?): Int
    fun MsiViewExecute(hView: Pointer?, hRecord: Pointer?): Int
    fun MsiViewFetch(hView: Pointer?, phRecord: PointerByReference?): Int
    fun MsiRecordGetString(hRecord: Pointer?, iField: Int, szValueBuf: CharArray?, pcchValueBuf: IntByReference?): Int
    fun MsiCloseHandle(hAny: Pointer?): Int
    fun MsiGetSummaryInformation(
        hDatabase: Pointer?,
        szDatabasePath: WString?,
        uiUpdateCount: Int,
        phSummaryInfo: PointerByReference?
    ): Int
    fun MsiSummaryInfoGetProperty(
        hSummaryInfo: Pointer,
        uiProperty: Int,
        puiDataType: IntByReference,
        piValue: IntByReference,
        pftValue: FILETIME,
        szValueBuf: CharArray,
        pcchValueBuf: IntByReference
    ): Int

    companion object {
        private const val msi = "msi"
        val INSTANCE: MsiLibrary = Native.load(msi, MsiLibrary::class.java, W32APIOptions.UNICODE_OPTIONS)
    }
}
