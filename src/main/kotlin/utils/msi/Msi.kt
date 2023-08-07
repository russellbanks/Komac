package utils.msi

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.platform.win32.WinError
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import okio.Path
import schemas.manifest.InstallerManifest
import utils.extension
import utils.jna.GObject
import utils.jna.LibMsi
import utils.jna.WinMsi

class Msi(private val msiFile: Path) {
    var productCode: String? = null
    var upgradeCode: String? = null
    var productName: String? = null
    var productVersion: String? = null
    var manufacturer: String? = null
    var productLanguage: String? = null
    var allUsers: AllUsers? = null
    var isWix: Boolean = false
    var architecture: InstallerManifest.Installer.Architecture? = null
    var description: String? = null

    private val sql = sqlQuery {
        select(PROPERTY, VALUE)
        from(PROPERTY)
        where(PROPERTY, values)
    }

    init {
        require(msiFile.extension.equals(InstallerManifest.InstallerType.MSI.name, ignoreCase = true))
        if (Platform.isWindows()) getValuesMsiDll() else getValuesLibMsi()
    }

    private fun getValuesLibMsi() {
        val libMsi = LibMsi.INSTANCE
        val gObject = GObject.INSTANCE
        val error = PointerByReference()

        val database = libMsi.libmsiDatabaseNew(msiFile.toString(), LibMsi.DB_FLAGS_READONLY, null, error)
        architecture = MsiArch(database).architecture

        val query = libMsi.libmsiQueryNew(database, sql, error)
        libMsi.libmsiQueryExecute(query, null, error)

        var rec = libMsi.libmsiQueryFetch(query, error)
        while (rec != null) {
            val property = libMsi.libmsiRecordGetString(rec, 1) ?: continue

            val value = libMsi.libmsiRecordGetString(rec, 2) ?: continue

            setValue(property, value)

            gObject.gObjectUnref(rec)
            rec = libMsi.libmsiQueryFetch(query, error)
        }
        if (error.value != null) {
            gObject.gClearError(error.pointer)
        }
        with(gObject) {
            gObjectUnref(query)
            gObjectUnref(database)
        }
    }

    private fun getValuesMsiDll() {
        val winMsi = WinMsi.INSTANCE
        val database = winMsi.openDatabase() ?: return

        architecture = MsiArch(database.value).architecture

        val view = winMsi.openView(database)

        if (view != null) {
            if (winMsi.executeView(view) == WinError.ERROR_SUCCESS) {
                winMsi.fetchRecords(view)
            }
            winMsi.MsiCloseHandle(view.value)
        }
        winMsi.MsiCloseHandle(database.value)
    }

    private fun WinMsi.openDatabase(): PointerByReference? {
        val phDatabase = PointerByReference()
        val result = MsiOpenDatabase(msiFile.toString(), WinMsi.MSI_DB_OPEN_READ_ONLY, phDatabase)
        if (result != WinError.ERROR_SUCCESS) {
            println("Error opening database: $result")
            return null
        }
        return phDatabase
    }

    private fun WinMsi.openView(phDatabase: PointerByReference): PointerByReference? {
        val phView = PointerByReference()
        val result = MsiDatabaseOpenView(phDatabase.value, sql, phView)
        if (result != WinError.ERROR_SUCCESS) {
            println("Error executing query: $result")
            return null
        }
        return phView
    }

    private fun WinMsi.executeView(phView: PointerByReference): Int {
        val result = MsiViewExecute(phView.value, null)
        if (result != WinError.ERROR_SUCCESS) {
            println("Error executing view: $result")
        }
        return result
    }

    private fun WinMsi.fetchRecords(phView: PointerByReference) {
        val phRecord = PointerByReference()
        while (true) {
            val result = MsiViewFetch(phView.value, phRecord)
            if (result != WinError.ERROR_SUCCESS) {
                break
            }

            val property = extractString(phRecord = phRecord, field = 1, bufferSize = PROPERTY_BUFFER_SIZE)
            val value = extractString(phRecord = phRecord, field = 2, bufferSize = VALUE_BUFFER_SIZE)

            setValue(property, value)

            MsiCloseHandle(phRecord.value)
        }
    }

    private fun WinMsi.extractString(phRecord: PointerByReference, field: Int, bufferSize: Int): String? {
        val pcchBuf = IntByReference()
        val szBuf = CharArray(bufferSize)
        pcchBuf.value = bufferSize

        val result = MsiRecordGetString(phRecord.value, field, szBuf, pcchBuf)
        return if (result == WinError.ERROR_SUCCESS) Native.toString(szBuf) else null
    }

    fun setValue(property: String?, value: String?) {
        when (property) {
            UPGRADE_CODE -> upgradeCode = value
            PRODUCT_CODE -> productCode = value
            PRODUCT_NAME -> productName = value
            PRODUCT_VERSION -> productVersion = value
            MANUFACTURER -> manufacturer = value
            PRODUCT_LANGUAGE -> productLanguage = value?.toIntOrNull()?.let { ProductLanguage(it).locale }
            WIXUI_MODE -> isWix = true
            ALL_USERS -> allUsers = AllUsers.entries.find { it.code == value }
        }
    }

    enum class AllUsers(val code: String) {
        Machine("1"),
        User(""),
        Dependent("2");

        fun toInstallerScope() = when (this) {
            Machine -> InstallerManifest.Scope.Machine
            User -> InstallerManifest.Scope.User
            Dependent -> null
        }
    }

    companion object {
        private const val PROPERTY = "Property"
        private const val VALUE = "Value"
        private const val UPGRADE_CODE = "UpgradeCode"
        private const val PRODUCT_CODE = "ProductCode"
        private const val PRODUCT_NAME = "ProductName"
        private const val PRODUCT_VERSION = "ProductVersion"
        private const val MANUFACTURER = "Manufacturer"
        private const val PRODUCT_LANGUAGE = "ProductLanguage"
        private const val WIXUI_MODE = "WixUI_Mode"
        private const val ALL_USERS = "ALLUSERS"
        private const val PROPERTY_BUFFER_SIZE = 64
        private const val VALUE_BUFFER_SIZE = 1024
        val values = listOf(
            UPGRADE_CODE,
            PRODUCT_CODE,
            PRODUCT_NAME,
            PRODUCT_VERSION,
            MANUFACTURER,
            PRODUCT_LANGUAGE,
            WIXUI_MODE,
            ALL_USERS
        )
    }
}
