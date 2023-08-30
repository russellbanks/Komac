package network

import io.ktor.client.HttpClient
import io.ktor.client.engine.winhttp.WinHttp
import io.ktor.http.HttpProtocolVersion

actual object Http {
    actual val client = HttpClient(WinHttp) {
        engine {
            pipelining = true
            protocolVersion = HttpProtocolVersion.QUIC
        }
        ClientModules.install(this)
    }
}
