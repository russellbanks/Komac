package schemas

import Ktor
import com.github.ajalt.mordant.animation.progressAnimation
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.utils.io.core.use
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

@Single
class InstallerSchemaImpl : KoinComponent {
    lateinit var installerSchema: InstallerSchema
    private val terminalInstance: TerminalInstance by inject()

    private var asyncJob: Deferred<Unit> = CoroutineScope(Dispatchers.Default).async {
        val json = Json { ignoreUnknownKeys = true }
        HttpClient(Java) {
            install(ContentNegotiation)
            install(UserAgent) {
                agent = Ktor.userAgent
            }
        }.use {
            installerSchema = json.decodeFromString(it.get(Schemas.installerSchema).body())
        }
    }

    suspend fun awaitInstallerSchema() {
        with(asyncJob) {
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
}
