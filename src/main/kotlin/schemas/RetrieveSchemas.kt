package schemas

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import org.koin.core.annotation.Single

@Single
class RetrieveSchemas {
    var client: HttpClient = HttpClient(Java) {
        install(ContentNegotiation)
        install(UserAgent) {
            agent = "Microsoft-Delivery-Optimization/10.1"
        }
    }
}