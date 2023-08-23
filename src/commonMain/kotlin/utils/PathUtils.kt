package utils

import okio.FileSystem
import okio.HashingSink.Companion.sha256
import okio.Path
import okio.blackholeSink
import okio.buffer
import okio.use

val Path.extension: String get() = name.substringAfterLast('.')

fun Path.hashSha256(fileSystem: FileSystem = io.FileSystem.SYSTEM): String {
    sha256(blackholeSink()).use { hashingSink ->
        fileSystem.source(this).buffer().use { source ->
            source.readAll(hashingSink)
            return hashingSink.hash.hex()
        }
    }
}
