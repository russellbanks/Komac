package detection.files.msi

import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.WString
import com.sun.jna.platform.win32.WinBase.FILETIME
import com.sun.jna.ptr.IntByReference
import com.sun.jna.ptr.PointerByReference
import extensions.extension
import okio.Buffer
import okio.ByteString
import okio.ByteString.Companion.encodeUtf8
import okio.FileSystem
import okio.Path
import schemas.manifest.InstallerManifest

class Msi(private val msiFile: Path, private val fileSystem: FileSystem) {
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

    private val msiLibrary = MsiLibrary.INSTANCE

    init {
        require(msiFile.extension == InstallerManifest.InstallerType.MSI.toString())
        if (Platform.isWindows()) getValuesFromDatabase() else getValuesFromBinary()
    }

    private fun getValuesFromDatabase() {
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
                        if (productLanguage == null) productLanguage = groupValues
                            ?.getOrNull(3)
                            ?.toIntOrNull()
                            ?.let(::ProductLanguage)
                            ?.locale
                        productName = groupValues?.getOrNull(4)
                        productVersion = groupValues?.getOrNull(5)
                        upgradeCode = groupValues?.getOrNull(6)
                        return
                    }
                }
            }
        }
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
                Native.toString(szBuf).split(';').first().toArchitecture()
            } else {
                null
            }
        } else {
            null
        }
    }

    private fun openDatabase(): PointerByReference? {
        val phDatabase = PointerByReference()
        val result = msiLibrary.MsiOpenDatabaseW(WString(msiFile.toString()), WString(msiDbOpenReadOnly), phDatabase)
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
                productLanguageConst -> productLanguage = value?.toIntOrNull()?.let { ProductLanguage(it).locale }
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

    private fun String.toArchitecture(): InstallerManifest.Installer.Architecture? {
        return when (this) {
            "x64", "Intel64", "AMD64" -> InstallerManifest.Installer.Architecture.X64
            "Intel" -> InstallerManifest.Installer.Architecture.X86
            else -> null
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
