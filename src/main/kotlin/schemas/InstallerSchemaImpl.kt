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
class InstallerSchemaImpl : KoinComponent {
    private val retrieveSchemas: RetrieveSchemas = get()
    private val client: HttpClient = retrieveSchemas.client
    var installerSchema: InstallerSchema? = null
    private val terminalInstance: TerminalInstance by inject()
    private val progress = terminalInstance.terminal.progressAnimation {
        text("Retrieving installer schema")
        progressBar()
    }

    init {
        CoroutineScope(Dispatchers.Default).launch {
            progress.run {
                start()
                client.get(Schemas.installerSchema) {
                    onDownload { bytesSentTotal, contentLength ->
                        progress.update(bytesSentTotal, contentLength)
                    }
                }.body<String?>()?.let {
                    val json = Json { ignoreUnknownKeys = true }
                    installerSchema = json.decodeFromString(it)
                }
                stop()
                clear()
            }
        }
    }

    val packageIdentifierPattern
        get() = installerSchema?.definitions?.packageIdentifier?.pattern?.toRegex() as Regex

    val packageIdentifierMaxLength
        get() = installerSchema?.definitions?.packageIdentifier?.maxLength as Int

    val packageVersionPattern
        get() = installerSchema?.definitions?.packageVersion?.pattern?.toRegex() as Regex

    val packageVersionMaxLength
        get() = installerSchema?.definitions?.packageVersion?.maxLength as Int

    val installerUrlPattern
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.pattern?.toRegex() as Regex

    val installerUrlMaxLength
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.maxLength as Int
}
