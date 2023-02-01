package detection.files.msi

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.WString
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import org.koin.core.component.KoinComponent
import schemas.manifest.InstallerManifest
import java.io.File

@Suppress("LoopWithTooManyJumpStatements")
class Msi(private val msiFile: File) : KoinComponent {
    var productCode: String? = null
    var upgradeCode: String? = null
    var productName: String? = null
    var productVersion: String? = null
    var manufacturer: String? = null
    var productLanguage: String? = null
    var allUsers: AllUsers? = null
    var isWix: Boolean = false
    var architecture: InstallerManifest.Installer.Architecture? = null

    private val msiLibrary = MsiLibrary.INSTANCE

    init {
        require(Platform.isWindows())
        getValues()
    }

    private fun getValues() {
        val phDatabase = openDatabase() ?: return

        architecture = getArchitecture(phDatabase)

        val phView = openView(phDatabase)

        if (phView != null) {
            if (executeView(phView) == 0) {
                fetchRecords(phView)
            }
            msiLibrary.MsiCloseHandle(phView.value)
        }
        msiLibrary.MsiCloseHandle(phDatabase.value)
    }

    private fun getArchitecture(phDatabase: PointerByReference): InstallerManifest.Installer.Architecture? {
        val phSummaryInfo = PointerByReference()
        var result = msiLibrary.MsiGetSummaryInformationW(phDatabase.value, null, 0, phSummaryInfo)
        return if (result == 0) {
            val pcchBuf = IntByReference()
            val szBuf = CharArray(16)
            pcchBuf.value = 16
            result = msiLibrary.MsiSummaryInfoGetPropertyW(
                hSummaryInfo = phSummaryInfo.value,
                uiProperty = architecturePropertyOrdinal,
                puiDataType = IntByReference(),
                piValue = IntByReference(),
                pftValue = FILETIME(),
                szValueBuf = szBuf,
                pcchValueBuf = pcchBuf
            )
            msiLibrary.MsiCloseHandle(phSummaryInfo.value)
            if (result == 0) {
                when (Native.toString(szBuf).split(";").first()) {
                    "x64", "Intel64", "AMD64" -> InstallerManifest.Installer.Architecture.X64
                    "Intel" -> InstallerManifest.Installer.Architecture.X86
                    else -> null
                }
            } else {
                null
            }
        } else {
            msiLibrary.MsiCloseHandle(phDatabase.value)
            null
        }
    }

    private fun openDatabase(): PointerByReference? {
        val phDatabase = PointerByReference()
        val result = msiLibrary.MsiOpenDatabaseW(WString(msiFile.path), WString(msiDbOpenReadOnly), phDatabase)
        if (result != 0) {
            println("Error opening database: $result")
            return null
        }
        return phDatabase
    }

    private fun openView(phDatabase: PointerByReference): PointerByReference? {
        val phView = PointerByReference()
        val result = msiLibrary.MsiDatabaseOpenViewW(
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
            return null
        }
        return phView
    }

    private fun executeView(phView: PointerByReference): Int {
        val result = msiLibrary.MsiViewExecute(phView.value, null)
        if (result != 0) {
            println("Error executing view: $result")
        }
        return result
    }

    private fun fetchRecords(phView: PointerByReference) {
        val phRecord = PointerByReference()
        while (true) {
            val result = msiLibrary.MsiViewFetch(phView.value, phRecord)
            if (result != 0) {
                break
            }

            val property = extractString(phRecord = phRecord, field = 1, bufferSize = propertyBufferSize)
            val value = extractString(phRecord = phRecord, field = 2, bufferSize = valueBufferSize)

            when (property) {
                upgradeCodeConst -> upgradeCode = value
                productCodeConst -> productCode = value
                productNameConst -> productName = value
                productVersionConst -> productVersion = value
                manufacturerConst -> manufacturer = value
                productLanguageConst -> productLanguage = ProductLanguage(value?.toIntOrNull()).locale
                wixUiModeConst -> isWix = true
                allUsersConst -> allUsers = AllUsers.values().find { it.code == value }
            }

            msiLibrary.MsiCloseHandle(phRecord.value)
        }
    }

    private fun extractString(phRecord: PointerByReference, field: Int, bufferSize: Int): String? {
        val pcchBuf = IntByReference()
        val szBuf = CharArray(bufferSize)
        pcchBuf.value = bufferSize

        val result = msiLibrary.MsiRecordGetStringW(phRecord.value, field, szBuf, pcchBuf)
        return if (result == 0) Native.toString(szBuf) else null
    }

    fun resetExceptShared() {
        productVersion = null
        productCode = null
        upgradeCode = null
        productLanguage = null
        allUsers = null
        isWix = false
    }

    enum class AllUsers(val code: String) {
        Machine("1"),
        User(""),
        Dependent("2");

        fun toInstallerScope() = when (this) {
            Machine -> InstallerManifest.Installer.Scope.Machine
            User -> InstallerManifest.Installer.Scope.User
            Dependent -> null
        }
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
        private const val propertyBufferSize = 64
        private const val valueBufferSize = 1024
        private const val architecturePropertyOrdinal = 7
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
