package utils

import com.appmattus.crypto.Algorithm
import com.appmattus.crypto.Digest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream
import java.io.InputStream

object Hashing {

    private const val hex255 = 0xff
    private const val hex256 = 0x100

    suspend fun File.hash(
        algorithm: Algorithm = Algorithm.SHA_256,
        hashProgressCallback: (Float) -> Unit = {}
    ): String {
        val digest = algorithm.createDigest()
        withContext(Dispatchers.IO) {
            FileInputStream(this@hash).use { fileInputStream ->
                val buffer = ByteArray(size = 32_768)
                var bytesCount: Int

                val totalRuns = (length() / buffer.size + 1).toFloat()
                var count = 0
                while (fileInputStream.read(buffer).also { bytesCount = it } != -1) {
                    digest.update(buffer, 0, bytesCount)
                    hashProgressCallback(++count / totalRuns)
                }
            }
        }
        return buildHash(digest.digest())
    }

    fun String.hash(algorithm: Algorithm) = buildHash(algorithm.createDigest().apply { update(toByteArray()) }.digest())

    fun updateDigest(inputStream: InputStream, digest: Digest<*>) {
        val buffer = ByteArray(size = 1_024)
        var bytesCount = inputStream.read(buffer)
        while (bytesCount > 0) {
            digest.update(buffer, 0, bytesCount)
            bytesCount = inputStream.read(buffer)
        }
    }

    fun buildHash(bytes: ByteArray) = buildString {
        bytes.forEach { byte ->
            append(((byte.toInt() and hex255) + hex256).toString(radix = 16).substring(startIndex = 1))
        }
    }
}
