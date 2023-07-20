package utils.jna

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer

interface GObject : Library {
    fun gObjectUnref(`object`: Pointer)

    fun gClearError(errorPtrPtr: Pointer)

    companion object {
        private const val GOBJECT = "gobject-2.0"
        val INSTANCE: GObject = Native.load(
            GOBJECT,
            GObject::class.java,
            mapOf(Library.OPTION_FUNCTION_MAPPER to JNAFunctionMapper.snakeCaseMapper)
        )
    }
}
