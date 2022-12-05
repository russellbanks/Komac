package hashing

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream
import java.io.IOException
import java.security.MessageDigest

object Hashing {

    private const val hex255 = 0xff
    private const val hex256 = 0x100

    @Throws(IOException::class, IllegalArgumentException::class, IllegalStateException::class)
    suspend fun File.hash(
        digest: MessageDigest,
        hashProgressCallback: (Float) -> Unit = {}
    ): String {
        val fileInputStream = withContext(Dispatchers.IO) { FileInputStream(this@hash) }

        val byteArray = ByteArray(size = 32_768)
        var bytesCount: Int

        val totalRuns = ((length() / byteArray.size) + 1).toFloat()
        var count = 0
        while (withContext(Dispatchers.IO) { fileInputStream.read(byteArray) }.also { bytesCount = it } != -1) {
            digest.update(byteArray, 0, bytesCount)
            hashProgressCallback(count++ / totalRuns)
        }
        hashProgressCallback(count / totalRuns)

        withContext(Dispatchers.IO) { fileInputStream.close() }

        return buildHash(digest.digest())
    }

    private fun buildHash(bytes: ByteArray) = buildString {
        bytes.indices.forEach { index ->
            append(((bytes[index].toInt() and hex255) + hex256).toString(radix = 16).substring(startIndex = 1))
        }
    }

    object Algorithms {
        private const val SHA_256 = "SHA-256"
        val SHA256: MessageDigest = MessageDigest.getInstance(SHA_256)
    }
}
