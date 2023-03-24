package network

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.UserAgent

/**
 * Stores the [HttpClient] instance and its configuration.
 */
object Http {
    /**
     * The [HttpClient] instance.
     *
     * This uses the [Java] engine
     */
    val client = HttpClient(Java) {
        engine {
            pipelining = true
        }
        install(UserAgent) {
            agent = userAgent
        }
        install(HttpTimeout) {
            connectTimeoutMillis = timeoutMillis
        }
    }

    private const val timeoutMillis = 1500L
    private const val userAgent = "Microsoft-Delivery-Optimization/10.1"
}
