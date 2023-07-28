package utils.msi

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import okio.Buffer
import okio.ByteString
import okio.ByteString.Companion.encodeUtf8
import okio.FileSystem
import okio.Path
import schemas.manifest.InstallerManifest
import utils.extension
import utils.jna.GObject
import utils.jna.LibMsi
import utils.jna.WinMsi
import utils.msi.MsiArch.Companion.toArchitecture

class Msi(private val msiFile: Path, private val fileSystem: FileSystem = FileSystem.SYSTEM) {
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
        select(property, value)
        from(property)
        where(property, values)
    }

    init {
        require(msiFile.extension.equals(InstallerManifest.InstallerType.MSI.name, ignoreCase = true))
        if (Platform.isWindows()) {
            getValuesWindows()
        } else if (Platform.isLinux()) {
            getValuesLinux()
        } else {
            getValuesFromBinary()
        }
    }

    private fun getValuesLinux() {
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

    private fun getValuesWindows() {
        val winMsi = WinMsi.INSTANCE
        val database = winMsi.openDatabase() ?: return

        architecture = MsiArch(database.value).architecture

        val view = winMsi.openView(database)

        if (view != null) {
            if (winMsi.executeView(view) == WinMsi.Errors.ERROR_SUCCESS) {
                winMsi.fetchRecords(view)
            }
            winMsi.MsiCloseHandle(view.value)
        }
        winMsi.MsiCloseHandle(database.value)
    }

    private fun getValuesFromBinary() {
        fileSystem.source(msiFile).use { source ->
            Buffer().use { buffer ->
                while (source.read(buffer, DEFAULT_BUFFER_SIZE.toLong()) != -1L) {
                    val byteString = buffer.readByteString()
                    val byteStringUtf8 = byteString.utf8()
                    if (byteStringUtf8.contains(installationDatabase)) {
                        val zeroByteString = ByteString.of(0)
                        val databaseBytes = installationDatabase.encodeUtf8()
                        val descriptionIndex = byteString.indexOf(databaseBytes) + databaseBytes.size + 11
                        val descriptionBytes = byteString
                            .substring(descriptionIndex, byteString.indexOf(zeroByteString, descriptionIndex))
                        description = descriptionBytes.utf8()
                        val manufacturerIndex = descriptionIndex + descriptionBytes.size + 9
                        manufacturer = byteString
                            .substring(manufacturerIndex, byteString.indexOf(zeroByteString, manufacturerIndex))
                            .utf8()
                        val semicolonIndex = byteString.indexOf(ByteString.of(';'.code.toByte()), manufacturerIndex)
                        var indexPointer = semicolonIndex
                        while (byteString[indexPointer.dec()] != 0.toByte()) indexPointer--
                        architecture = byteString.substring(indexPointer, semicolonIndex).utf8().toArchitecture()
                        indexPointer = semicolonIndex
                        while (byteString[indexPointer] != 0.toByte()) indexPointer++
                        if (productLanguage == null) {
                            productLanguage = byteString
                                .substring(semicolonIndex.inc(), indexPointer)
                                .utf8()
                                .toIntOrNull()
                                .takeIf { it != 0 }
                                ?.let(::ProductLanguage)
                                ?.locale
                        }
                    }
                    if (byteStringUtf8.contains(fullRegex)) {
                        if (byteStringUtf8.contains(other = wix, ignoreCase = true)) isWix = true
                        val groupValues = fullRegex.find(byteStringUtf8)?.groupValues?.map { it.ifBlank { null } }
                        if (manufacturer == null) manufacturer = groupValues?.getOrNull(1)
                        productCode = groupValues?.getOrNull(2)
                        if (productLanguage == null) {
                            productLanguage = groupValues
                                ?.getOrNull(3)
                                ?.toIntOrNull()
                                ?.let(::ProductLanguage)
                                ?.locale
                        }
                        productName = groupValues?.getOrNull(4)
                        productVersion = groupValues?.getOrNull(5)
                        upgradeCode = groupValues?.getOrNull(6)
                        return
                    }
                }
            }
        }
    }

    private fun WinMsi.openDatabase(): PointerByReference? {
        val phDatabase = PointerByReference()
        val result = MsiOpenDatabase(msiFile.toString(), WinMsi.MSI_DB_OPEN_READ_ONLY, phDatabase)
        if (result != WinMsi.Errors.ERROR_SUCCESS) {
            println("Error opening database: $result")
            return null
        }
        return phDatabase
    }

    private fun WinMsi.openView(phDatabase: PointerByReference): PointerByReference? {
        val phView = PointerByReference()
        val result = MsiDatabaseOpenView(phDatabase.value, sql, phView)
        if (result != WinMsi.Errors.ERROR_SUCCESS) {
            println("Error executing query: $result")
            return null
        }
        return phView
    }

    private fun WinMsi.executeView(phView: PointerByReference): Int {
        val result = MsiViewExecute(phView.value, null)
        if (result != WinMsi.Errors.ERROR_SUCCESS) {
            println("Error executing view: $result")
        }
        return result
    }

    private fun WinMsi.fetchRecords(phView: PointerByReference) {
        val phRecord = PointerByReference()
        while (true) {
            val result = MsiViewFetch(phView.value, phRecord)
            if (result != WinMsi.Errors.ERROR_SUCCESS) {
                break
            }

            val property = extractString(phRecord = phRecord, field = 1, bufferSize = propertyBufferSize)
            val value = extractString(phRecord = phRecord, field = 2, bufferSize = valueBufferSize)

            setValue(property, value)

            MsiCloseHandle(phRecord.value)
        }
    }

    private fun WinMsi.extractString(phRecord: PointerByReference, field: Int, bufferSize: Int): String? {
        val pcchBuf = IntByReference()
        val szBuf = CharArray(bufferSize)
        pcchBuf.value = bufferSize

        val result = MsiRecordGetString(phRecord.value, field, szBuf, pcchBuf)
        return if (result == WinMsi.Errors.ERROR_SUCCESS) Native.toString(szBuf) else null
    }

    fun setValue(property: String?, value: String?) {
        when (property) {
            upgradeCodeConst -> upgradeCode = value
            productCodeConst -> productCode = value
            productNameConst -> productName = value
            productVersionConst -> productVersion = value
            manufacturerConst -> manufacturer = value
            productLanguageConst -> productLanguage = value?.toIntOrNull()?.let { ProductLanguage(it).locale }
            wixUiModeConst -> isWix = true
            allUsersConst -> allUsers = AllUsers.entries.find { it.code == value }
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
        private const val propertyBufferSize = 64
        private const val valueBufferSize = 1024
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

        private const val wix = "Wix"
        private const val productCodeRegex =
            "\\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\\}"
        private val fullRegex = buildString {
            append(manufacturerConst)
            append("(.*?)")
            append(productCodeConst)
            append("($productCodeRegex)")
            append(productLanguageConst)
            append("(\\d{0,6})")
            append(productNameConst)
            append("(.*?)")
            append(productVersionConst)
            append("(.*?)")
            append("($productCodeRegex)")
        }.toRegex()
        private const val installationDatabase = "Installation Database"
    }
}
