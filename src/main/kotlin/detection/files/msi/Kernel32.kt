package detection.files.msi

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.win32.W32APIOptions

@Suppress("FunctionName", "FunctionParameterNaming", "LocalVariableName")
interface Kernel32 : Library {
    fun GetLocaleInfo(Locale: Int, LCType: Int, lpLCData: CharArray?, cchData: Int): Int

    companion object {
        private const val kernel32 = "kernel32"
        val INSTANCE: Kernel32 = Native.load(kernel32, Kernel32::class.java, W32APIOptions.UNICODE_OPTIONS)
    }
}
