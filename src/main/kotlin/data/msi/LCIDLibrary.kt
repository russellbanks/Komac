package data.msi

import com.sun.jna.Library
import com.sun.jna.Native

@Suppress("FunctionName", "FunctionParameterNaming")
interface LCIDLibrary : Library {
    fun GetLocaleInfoW(Locale: Int, LCType: Int, lpLCData: CharArray?, cchData: Int): Int

    companion object {
        val INSTANCE = Native.load("kernel32", LCIDLibrary::class.java) as LCIDLibrary
    }
}