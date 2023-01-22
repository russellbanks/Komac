package schemas

import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import ktor.Clients
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.data.DefaultLocaleSchema
import schemas.data.InstallerSchema
import schemas.data.LocaleSchema
import schemas.data.VersionSchema

@Single
class SchemasImpl : KoinComponent {
    private val client: HttpClient = get<Clients>().httpClient
    private val json = Json { ignoreUnknownKeys = true }
    private var installerSchemaJob = CoroutineScope(Dispatchers.IO).launch {
        installerSchema = json.decodeFromString(client.get(Schemas.installerSchema).body())
    }
    private var defaultLocaleSchemaJob = CoroutineScope(Dispatchers.IO).launch {
        defaultLocaleSchema = json.decodeFromString(client.get(Schemas.defaultLocaleSchema).body())
    }
    private var localeSchemaJob = CoroutineScope(Dispatchers.IO).launch {
        localeSchema = json.decodeFromString(client.get(Schemas.localeSchema).body())
    }
    private var versionSchemaJob = CoroutineScope(Dispatchers.IO).launch {
        versionSchema = json.decodeFromString(client.get(Schemas.versionSchema).body())
    }
    lateinit var installerSchema: InstallerSchema
    lateinit var defaultLocaleSchema: DefaultLocaleSchema
    lateinit var localeSchema: LocaleSchema
    lateinit var versionSchema: VersionSchema
    var manifestOverride: String? = null

    suspend fun awaitSchema(schema: Schema, terminal: Terminal) {
        val job = when (schema) {
            Schema.Installer -> installerSchemaJob
            Schema.DefaultLocale -> defaultLocaleSchemaJob
            Schema.Locale -> localeSchemaJob
            Schema.Version -> versionSchemaJob
        }
        with(job) {
            if (isActive) {
                terminal.progressAnimation {
                    text("Retrieving $schema schema")
                    progressBar()
                }.run {
                    start()
                    invokeOnCompletion {
                        stop()
                        clear()
                    }
                    join()
                }
            }
        }
    }
}
