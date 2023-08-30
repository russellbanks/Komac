package network

import io.ktor.client.HttpClient
import io.ktor.client.engine.curl.Curl

actual object Http {
    actual val client = HttpClient(Curl) {
        engine {
            pipelining = true
        }
        ClientModules.install(this)
    }
}
