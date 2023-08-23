package network

import io.ktor.client.HttpClient
import io.ktor.client.engine.darwin.Darwin

actual object Http {
    actual val client = HttpClient(Darwin) {
        engine {
            pipelining = true
        }
        ClientModules.install(this)
    }
}
