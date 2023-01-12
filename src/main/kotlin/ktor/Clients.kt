package ktor

import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.UserAgent
import org.koin.core.annotation.Single

@Single
class Clients {
    val httpClient = HttpClient(Java) {
        install(UserAgent) {
            agent = Ktor.userAgent
        }
        install(HttpTimeout) {
            connectTimeoutMillis = timeoutMillis
        }
    }

    companion object {
        const val timeoutMillis = 1000L
    }
}
