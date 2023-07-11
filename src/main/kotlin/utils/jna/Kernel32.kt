package utils.jna

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.win32.W32APIOptions

@Suppress("FunctionName", "FunctionParameterNaming", "LocalVariableName")
interface Kernel32 : Library {
    fun GetLocaleInfo(Locale: Int, LCType: Int, lpLCData: CharArray?, cchData: Int): Int

    companion object {
        const val RT_RCDATA = "10"
        const val PRODUCT_CODE_RESOURCE = "PRODUCT_CODE"
        const val MSI_RESOURCE = "MSI"

        const val RT_MANIFEST = "24"
        const val MANIFEST_RESOURCE = "1"

        private const val kernel32 = "kernel32"
        val INSTANCE: Kernel32 = Native.load(kernel32, Kernel32::class.java, W32APIOptions.UNICODE_OPTIONS)
    }
}
