package utils

import kotlin.experimental.ExperimentalNativeApi
import kotlin.native.Platform

@OptIn(ExperimentalNativeApi::class)
actual object Platform {
    actual fun isWindows(): Boolean = Platform.osFamily == OsFamily.WINDOWS

    actual fun isLinux(): Boolean = Platform.osFamily == OsFamily.LINUX
}
