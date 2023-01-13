package data.msi

import com.sun.jna.Native
import com.sun.jna.WString
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.TerminalInstance
import java.io.File

@Suppress("LoopWithTooManyJumpStatements")
class Msi(private val msiFile: File) : KoinComponent {
    var productCode: String? = null
    var upgradeCode: String? = null
    var productName: String? = null
    var productVersion: String? = null
    var manufacturer: String? = null
    var productLanguage: Int? = null
    var allUsers: String? = null
    var isWix: Boolean = false

    private val msiLibrary = MsiLibrary.INSTANCE

    init {
        getValues()
    }

    private fun getValues() {
        with(get<TerminalInstance>().terminal) {
            val phDatabase = PointerByReference()
            var result = msiLibrary.MsiOpenDatabaseW(WString(msiFile.path), WString(msiDbOpenReadOnly), phDatabase)
            if (result != 0) {
                println("Error opening database: $result")
                return
            }

            val phView = PointerByReference()
            result = msiLibrary.MsiDatabaseOpenViewW(
                phDatabase.value,
                WString(
                    SQLBuilder()
                        .select(property, value)
                        .from(property)
                        .where(property, values)
                        .toString()
                ),
                phView
            )
            if (result != 0) {
                println("Error executing query: $result")
                msiLibrary.MsiCloseHandle(phDatabase.value)
                return
            }

            result = msiLibrary.MsiViewExecute(phView.value, null)
            if (result != 0) {
                println("Error executing view: $result")
                msiLibrary.MsiCloseHandle(phView.value)
                msiLibrary.MsiCloseHandle(phDatabase.value)
                return
            }

            val phRecord = PointerByReference()
            while (true) {
                result = msiLibrary.MsiViewFetch(phView.value, phRecord)
                if (result != 0) {
                    break
                }

                val pcchPropertyBuf = IntByReference()
                val szPropertyBuf = CharArray(propertyBufferSize)
                pcchPropertyBuf.value = propertyBufferSize
               result = msiLibrary.MsiRecordGetStringW(phRecord.value, 1, szPropertyBuf, pcchPropertyBuf)
                if (result != 0) {
                    println("Error getting property: $result")
                    msiLibrary.MsiCloseHandle(phRecord.value)
                    continue
                }
                val property = Native.toString(szPropertyBuf)

                val pcchValueBuf = IntByReference()
                val szValueBuf = CharArray(valueBufferSize)
                pcchValueBuf.value = valueBufferSize
                result = msiLibrary.MsiRecordGetStringW(phRecord.value, 2, szValueBuf, pcchValueBuf)
                if (result != 0) {
                    println("Error getting value: $result")
                    msiLibrary.MsiCloseHandle(phRecord.value)
                    continue
                }

                val value = Native.toString(szValueBuf)
                when (property) {
                    upgradeCodeConst -> upgradeCode = value
                    productCodeConst -> productCode = value
                    productNameConst -> productName = value
                    productVersionConst -> productVersion = value
                    manufacturerConst -> manufacturer = value
                    productLanguageConst -> productLanguage = value.toIntOrNull()
                    wixUiModeConst -> isWix = true
                    allUsersConst -> allUsers = value
                }
                msiLibrary.MsiCloseHandle(phRecord.value)
            }
            msiLibrary.MsiCloseHandle(phView.value)
            msiLibrary.MsiCloseHandle(phDatabase.value)
        }
    }

    fun resetExceptShared() {
        productCode = null
        upgradeCode = null
        productLanguage = null
        allUsers = null
        isWix = false
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
        private const val wixUiModeConst = "WixUI_Mode"
        private const val allUsersConst = "ALLUSERS"
        private const val msiDbOpenReadOnly = "MSIDBOPEN_READONLY"
        private const val propertyBufferSize = 16 // Length of "ProductLanguage" + null terminator
        private const val valueBufferSize = 39 // Length of ProductCode/UpgradeCode + null terminator
        val values = listOf(
            upgradeCodeConst,
            productCodeConst,
            productNameConst,
            productVersionConst,
            manufacturerConst,
            productLanguageConst,
            wixUiModeConst,
            allUsersConst
        )
    }
}
