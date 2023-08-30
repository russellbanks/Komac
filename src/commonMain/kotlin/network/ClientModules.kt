package network

import io.ktor.client.HttpClientConfig
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.UserAgent

object ClientModules {
    fun install(config: HttpClientConfig<*>) = config.apply {
        install(UserAgent) {
            agent = DELIVERY_OPTIMIZATION_USER_AGENT
        }
        install(HttpTimeout) {
            connectTimeoutMillis = TIMEOUT_MILLIS
        }
    }

    private const val TIMEOUT_MILLIS = 1500L
    private const val DELIVERY_OPTIMIZATION_USER_AGENT = "Microsoft-Delivery-Optimization/10.1"
}
