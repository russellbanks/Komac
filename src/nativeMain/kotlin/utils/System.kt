package utils

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.toKString

actual object System {
    @OptIn(ExperimentalForeignApi::class)
    actual fun getenv(name: String): String? = platform.posix.getenv(name)?.toKString()
}
