package ktor

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.UserAgent
import org.koin.core.annotation.Single

/**
 * Stores the [HttpClient] instance and its configuration.
 */
@Single
class Http {
    /**
     * The [HttpClient] instance.
     *
     * This uses the [Java] engine for performance and support for HTTP 2.
     */
    val client = HttpClient(Java) {
        engine {
            pipelining = true
            protocolVersion = java.net.http.HttpClient.Version.HTTP_2
        }
        install(UserAgent) {
            agent = userAgent
        }
        install(HttpTimeout) {
            connectTimeoutMillis = timeoutMillis
        }
    }

    companion object {
        private const val timeoutMillis = 1500L
        private const val userAgent = "Microsoft-Delivery-Optimization/10.1"
    }
}
