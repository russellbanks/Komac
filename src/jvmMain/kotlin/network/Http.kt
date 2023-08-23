package network

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java

/**
 * Stores the [HttpClient] instance and its configuration.
 */
actual object Http {
    actual val client = HttpClient(Java) {
        engine {
            pipelining = true
            protocolVersion = java.net.http.HttpClient.Version.HTTP_2
        }
        ClientModules.install(this)
    }
}
