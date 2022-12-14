import io.kotest.common.runBlocking
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import schemas.InstallerSchema
import schemas.Schemas

object InstallerSchemaData {
    val installerSchema: InstallerSchema = runBlocking {
        HttpClient(Java) {
            install(ContentNegotiation)
            install(UserAgent) {
                agent = Ktor.userAgent
            }
        }.use {
            val json = Json { ignoreUnknownKeys = true }
            json.decodeFromString(it.get(Schemas.installerSchema).body())
        }
    }
}
