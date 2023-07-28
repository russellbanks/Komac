package utils.jna

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference

@Suppress("FunctionName", "LongParameterList")
interface LibMsi : Library {
    fun libmsiDatabaseNew(path: String, flags: Int, persist: String?, error: PointerByReference): Pointer

    fun libmsiQueryNew(database: Pointer, query: String, error: PointerByReference): Pointer

    fun libmsiQueryFetch(query: Pointer, error: PointerByReference): Pointer?

    fun libmsiRecordGetString(record: Pointer, field: Int): String?

    fun libmsiQueryExecute(query: Pointer, rec: Pointer?, error: PointerByReference): Boolean

    fun libmsiSummaryInfoNew(database: Pointer, updateCount: Int, error: PointerByReference): Pointer

    fun libmsiSummaryInfoGetString(summaryInformation: Pointer, prop: Int, error: PointerByReference): String

    companion object {
        private const val LIBMSI = "libmsi"
        const val DB_FLAGS_READONLY = 1
        val INSTANCE: LibMsi = Native.load(
            LIBMSI,
            LibMsi::class.java,
            mapOf(Library.OPTION_FUNCTION_MAPPER to JNAFunctionMapper.snakeCaseMapper)
        )
    }
}
