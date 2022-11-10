package schemas

import com.github.ajalt.mordant.animation.progressAnimation
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.onDownload
import io.ktor.client.request.get
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject

@Single
class VersionSchemaImpl : KoinComponent {
    private val retrieveSchemas: RetrieveSchemas = get()
    private val client: HttpClient = retrieveSchemas.client
    var versionSchema: VersionSchema? = null
    private val terminalInstance: TerminalInstance by inject()
    private val progress = terminalInstance.terminal.progressAnimation {
        text("Retrieving version schema")
        progressBar()
    }

    init {
        CoroutineScope(Dispatchers.Default).launch {
            progress.run {
                start()
                client.get(Schemas.versionSchema) {
                    onDownload { bytesSentTotal, contentLength ->
                        progress.update(bytesSentTotal, contentLength)
                    }
                }.body<String?>()?.let {
                    val json = Json { ignoreUnknownKeys = true }
                    versionSchema = json.decodeFromString(it)
                }
                stop()
                clear()
            }
        }
    }

    fun packageIdentifier() = versionSchema?.properties?.packageIdentifier?.pattern?.toRegex() as Regex
    fun packageIdentifierMaxLength() = versionSchema?.properties?.packageIdentifier?.maxLength as Int
    fun packageVersion() = versionSchema?.properties?.packageVersion?.pattern?.toRegex() as Regex
    fun packageVersionMaxLength() = versionSchema?.properties?.packageVersion?.maxLength as Int
}