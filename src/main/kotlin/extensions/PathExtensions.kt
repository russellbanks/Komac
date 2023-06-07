package extensions

import okio.FileSystem
import okio.HashingSink.Companion.sha256
import okio.Path
import okio.blackholeSink
import okio.buffer

val Path.extension: String get() = name.substringAfterLast('.')

fun Path.hashSha256(fileSystem: FileSystem = FileSystem.SYSTEM): String {
    sha256(blackholeSink()).use { hashingSink ->
        fileSystem.source(this).buffer().use { source ->
            source.readAll(hashingSink)
            return hashingSink.hash.hex()
        }
    }
}
