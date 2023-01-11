package msi

import com.github.ajalt.mordant.terminal.Terminal
import com.sun.jna.Native
import com.sun.jna.WString
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.TerminalInstance
import java.io.File

data class Msi(val msiFile: File) : KoinComponent {
    var productCode: String? = null
    var upgradeCode: String? = null
    var productName: String? = null
    var productVersion: String? = null
    var manufacturer: String? = null
    var productLanguage: Int? = null
    var isWix: Boolean = false

    private val msiLibrary = MsiLibrary.INSTANCE

    init {
        getValues()
    }

    private fun getValues() {
        with(get<TerminalInstance>().terminal) {
            val database = openDatabase() ?: return

            val view = executeQuery(
                database = database,
                query = SQLBuilder()
                    .select(property, value)
                    .from(property)
                    .where(property, values)
                    .toString()
            ) ?: return

            executeView(database = database, view = view) ?: return

            // Fetch the records
            var result: Int
            val hRecord = LongArray(1)
            while (true) {
                result = msiLibrary.MsiViewFetch(view, hRecord)
                if (result != 0) {
                    break
                }
                val record = hRecord.first()

                // Get the property and value
                val szPropertyBuf = CharArray(1024)
                val pcchPropertyBuf = intArrayOf(szPropertyBuf.size)
                result = msiLibrary.MsiRecordGetStringW(record, 1, szPropertyBuf, pcchPropertyBuf)
                if (result == 0) {
                    val property = Native.toString(szPropertyBuf)

                    val szValueBuf = CharArray(1024)
                    val pcchValueBuf = intArrayOf(szValueBuf.size)
                    result = msiLibrary.MsiRecordGetStringW(record, 2, szValueBuf, pcchValueBuf)
                    if (result == 0) {
                        val value = Native.toString(szValueBuf)
                        when (property) {
                            upgradeCodeConst -> upgradeCode = value
                            productCodeConst -> productCode = value
                            productNameConst -> productName = value
                            productVersionConst -> productVersion = value
                            manufacturerConst -> manufacturer = value
                            productLanguageConst -> productLanguage = value.toIntOrNull()
                            wixUiMode -> isWix = true
                        }
                        msiLibrary.MsiCloseHandle(record)
                    } else {
                        msiLibrary.MsiCloseHandle(record)
                    }
                } else {
                    msiLibrary.MsiCloseHandle(record)
                }
            }

            // Close the view and database
            msiLibrary.MsiViewClose(view)
            msiLibrary.MsiCloseHandle(database)
        }
    }

    private fun Terminal.openDatabase(): Long? {
        return LongArray(1)
            .let {
                val result = msiLibrary.MsiOpenDatabaseW(WString(msiFile.path), 0, it)
                if (result != 0) {
                    println("Error opening database: $result")
                    null
                } else {
                    it
                }
            }
            ?.first()
    }

    private fun Terminal.executeQuery(database: Long, query: String): Long? {
        return LongArray(1)
            .let {
                val result = msiLibrary.MsiDatabaseOpenViewW(database, WString(query), it)
                if (result != 0) {
                    println("Error executing query: $result")
                    msiLibrary.MsiCloseHandle(database)
                    null
                } else {
                    it
                }
            }
            ?.first()
    }

    private fun Terminal.executeView(database: Long, view: Long): Unit? {
        val result = msiLibrary.MsiViewExecute(view, 0)
        return if (result != 0) {
            println("Error executing view: $result")
            msiLibrary.MsiViewClose(view)
            msiLibrary.MsiCloseHandle(database)
            null
        } else {
            Unit
        }
    }

    fun resetExceptShared() {
        productCode = null
        upgradeCode = null
        productLanguage = null
    }

    companion object {
        private const val property = "Property"
        private const val value = "Value"
        private const val upgradeCodeConst = "UpgradeCode"
        private const val productCodeConst = "ProductCode"
        private const val productNameConst = "ProductName"
        private const val productVersionConst = "ProductVersion"
        private const val manufacturerConst = "Manufacturer"
        private const val productLanguageConst = "ProductLanguage"
        private const val wixUiMode = "WixUI_Mode"
        val values = listOf(
            upgradeCodeConst,
            productCodeConst,
            productNameConst,
            productVersionConst,
            manufacturerConst,
            productLanguageConst,
            wixUiMode
        )
    }
}
