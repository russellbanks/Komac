package extensions

import okio.FileSystem
import okio.HashingSink.Companion.sha256
import okio.Path
import okio.blackholeSink
import okio.buffer

object PathExtensions {
    val Path.extension: String
        get() = name.substringAfterLast('.')

    fun Path.hash(fileSystem: FileSystem): String {
        sha256(blackholeSink()).use { hashingSink ->
            fileSystem.source(this).buffer().use { source ->
                source.readAll(hashingSink)
                return hashingSink.hash.hex()
            }
        }
    }
}
