package data.msi

import com.sun.jna.Native
import com.sun.jna.Platform

class ProductLanguage(languageCode: Int?) {
    val locale = languageCode?.toLanguageTag()

    init {
        require(Platform.isWindows())
    }

    private fun Int.toLanguageTag(): String? {
        val lcidLibrary = LCIDLibrary.INSTANCE

        val lpLanguage = CharArray(localeBufferSize)
        var result: Int = lcidLibrary.GetLocaleInfoW(this, LOCALE_SABBREVLANGNAME, lpLanguage, lpLanguage.size)
        if (result > 0) {
            val language = Native.toString(lpLanguage).dropLast(1)
            val lpCountry = CharArray(localeBufferSize)
            result = lcidLibrary.GetLocaleInfoW(this, LOCALE_SABBREVCTRYNAME, lpCountry, lpCountry.size)
            if (result > 0) {
                val country = Native.toString(lpCountry).dropLast(1)
                return language.lowercase() + "-" + country.uppercase()
            }
        }
        return null
    }

    companion object {
        private const val localeBufferSize = 4 // 3 Characters (e.g. USA) + null terminator
        private const val LOCALE_SABBREVLANGNAME = 0x3
        private const val LOCALE_SABBREVCTRYNAME = 0x7
    }
}
