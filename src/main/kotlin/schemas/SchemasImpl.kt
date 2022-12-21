package schemas

import com.github.ajalt.mordant.animation.progressAnimation
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.get
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import ktor.Ktor
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

@Single
class SchemasImpl : KoinComponent {
    private val terminalInstance: TerminalInstance by inject()
    private lateinit var installerSchemaJob: Deferred<Unit>
    private lateinit var defaultLocaleSchemaJob: Deferred<Unit>
    private lateinit var localeSchemaJob: Deferred<Unit>
    private lateinit var versionSchemaJob: Deferred<Unit>
    lateinit var installerSchema: InstallerSchema
    lateinit var defaultLocaleSchema: DefaultLocaleSchema
    lateinit var localeSchema: LocaleSchema
    lateinit var versionSchema: VersionSchema

    init {
        CoroutineScope(Dispatchers.Default).launch {
            val client = HttpClient(Java) {
                install(UserAgent) {
                    agent = Ktor.userAgent
                }
            }
            val json = Json { ignoreUnknownKeys = true }
            installerSchemaJob = async {
                installerSchema = json.decodeFromString(client.get(Schemas.installerSchema).body())
            }
            defaultLocaleSchemaJob = async {
                defaultLocaleSchema = json.decodeFromString(client.get(Schemas.defaultLocaleSchema).body())
            }
            localeSchemaJob = async {
                localeSchema = json.decodeFromString(client.get(Schemas.localeSchema).body())
            }
            versionSchemaJob = async {
                versionSchema = json.decodeFromString(client.get(Schemas.versionSchema).body())
            }
            installerSchemaJob.await()
            defaultLocaleSchemaJob.await()
            localeSchemaJob.await()
            versionSchemaJob.await()
            client.close()
        }
    }

    suspend fun awaitSchema(schema: Schema) {
        val job = when (schema) {
            Schema.Installer -> installerSchemaJob
            Schema.DefaultLocale -> defaultLocaleSchemaJob
            Schema.Locale -> localeSchemaJob
            Schema.Version -> versionSchemaJob
        }
        with(job) {
            if (isActive) {
                terminalInstance.terminal.progressAnimation {
                    text("Retrieving $schema schema")
                    progressBar()
                }.run {
                    start()
                    invokeOnCompletion {
                        stop()
                        clear()
                    }
                    await()
                }
            }
        }
    }
}
