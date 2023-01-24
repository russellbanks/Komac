package ktor

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.UserAgent
import org.koin.core.annotation.Single

@Single
class Clients {
    val httpClient = HttpClient(Java) {
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
        const val timeoutMillis = 1500L
        const val userAgent = "Microsoft-Delivery-Optimization/10.1"
    }
}
