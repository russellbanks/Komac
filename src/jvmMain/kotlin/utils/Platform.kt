package utils

import com.sun.jna.Platform

actual object Platform {
    actual fun isWindows(): Boolean = Platform.isWindows()

    actual fun isLinux(): Boolean = Platform.isLinux()
}
