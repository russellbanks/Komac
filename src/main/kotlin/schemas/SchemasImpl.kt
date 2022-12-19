package schemas

import Ktor
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
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

@Single
class SchemasImpl : KoinComponent {
    private val terminalInstance: TerminalInstance by inject()
    lateinit var installerSchema: InstallerSchema
    private lateinit var installerSchemaJob: Deferred<Unit>
    lateinit var defaultLocaleSchema: DefaultLocaleSchema
    private lateinit var defaultLocaleSchemaJob: Deferred<Unit>

    init {
        val client = HttpClient(Java) {
            install(UserAgent) {
                agent = Ktor.userAgent
            }
        }
        val json = Json { ignoreUnknownKeys = true }
        CoroutineScope(Dispatchers.Default).launch {
            installerSchemaJob = async {
                installerSchema = json.decodeFromString(client.get(Schemas.installerSchema).body())
            }
            installerSchemaJob.await()
            defaultLocaleSchemaJob = async {
                defaultLocaleSchema = json.decodeFromString(client.get(Schemas.defaultLocaleSchema).body())
            }
            defaultLocaleSchemaJob.await()
            client.close()
        }
    }

    suspend fun awaitInstallerSchema() {
        with(installerSchemaJob) {
            if (isActive) {
                terminalInstance.terminal.progressAnimation {
                    text("Retrieving installer schema")
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

    suspend fun awaitDefaultLocaleSchema() {
        with(defaultLocaleSchemaJob) {
            if (isActive) {
                terminalInstance.terminal.progressAnimation {
                    text("Retrieving default locale schema")
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
