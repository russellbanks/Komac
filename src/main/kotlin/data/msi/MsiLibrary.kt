package data.msi

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.WString
import schemas.InstallerManifest

@Suppress("FunctionName")
interface MsiLibrary : Library {
    fun MsiOpenDatabaseW(szDatabasePath: WString?, phPersist: Long, phDatabase: LongArray?): Int
    fun MsiDatabaseOpenViewW(hDatabase: Long, szQuery: WString?, phView: LongArray?): Int
    fun MsiViewExecute(hView: Long, hRecord: Long): Int
    fun MsiViewFetch(hView: Long, hRecord: LongArray?): Int
    fun MsiRecordGetStringW(hRecord: Long, iField: Int, szValueBuf: CharArray?, pcchValueBuf: IntArray?): Int
    fun MsiCloseHandle(hAny: Long): Int
    fun MsiViewClose(hView: Long): Int

    companion object {
        val INSTANCE = Native.load(InstallerManifest.InstallerType.MSI.toString(), MsiLibrary::class.java) as MsiLibrary
    }
}
