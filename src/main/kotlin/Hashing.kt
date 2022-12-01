import com.appmattus.crypto.Algorithm
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream
import java.io.IOException

object Hashing {

    @Throws(IOException::class, IllegalArgumentException::class, IllegalStateException::class)
    suspend fun File.hash(
        algorithm: Algorithm,
        hashProgressCallback: (Float) -> Unit = {}
    ): String {
        val digest = algorithm.createDigest()
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

    private fun buildHash(bytes: ByteArray) = StringBuilder().apply {
        bytes.indices.forEach { index ->
            append(((bytes[index].toInt() and 0xff) + 0x100).toString(radix = 16).substring(startIndex = 1))
        }
    }.toString()
}
