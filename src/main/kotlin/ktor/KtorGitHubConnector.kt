package ktor

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.headers
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.HttpMethod
import io.ktor.http.contentType
import io.ktor.util.toMap
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.connector.GitHubConnector
import org.kohsuke.github.connector.GitHubConnectorRequest
import org.kohsuke.github.connector.GitHubConnectorResponse
import org.kohsuke.github.connector.GitHubConnectorResponse.ByteArrayResponse
import java.io.IOException
import java.io.InputStream

class KtorGitHubConnector(private val client: HttpClient) : GitHubConnector {
    @Throws(IOException::class)
    override fun send(connectorRequest: GitHubConnectorRequest): GitHubConnectorResponse = runBlocking {
        val request = client.request(connectorRequest.url()) {
            method = HttpMethod.parse(connectorRequest.method())
            if (connectorRequest.hasBody()) {
                setBody(connectorRequest.body())
            }
            headers {
                connectorRequest.allHeaders().forEach { (key: String, value: List<String>) ->
                    appendAll(key, value)
                }
            }
            connectorRequest.contentType()?.let {
                contentType(ContentType.parse(it))
            }
        }
        KtorGitHubConnectorResponse(connectorRequest, request)
    }

    private class KtorGitHubConnectorResponse(
        request: GitHubConnectorRequest,
        private val response: HttpResponse
    ) : ByteArrayResponse(request, response.status.value, response.headers.toMap()) {
        override fun rawBodyStream(): InputStream? = runBlocking {
            return@runBlocking response.body()
        }
    }
}
