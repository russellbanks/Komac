package network

import io.ktor.client.HttpClient

expect object Http {
    val client: HttpClient
}
