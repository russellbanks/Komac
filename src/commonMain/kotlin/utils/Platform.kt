package utils

expect object Platform {
    fun isWindows(): Boolean

    fun isLinux(): Boolean
}