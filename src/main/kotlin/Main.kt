import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.NoTransformationFoundException
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromJsonElement
import schemas.InstallerSchema
import schemas.Schema
import schemas.Schemas
import schemas.VersionSchema
import kotlin.system.exitProcess

suspend fun main() {
    fun optionBox(char: Char): String {
        return "${blue("[")}${brightWhite(char.toString())}${blue("]")}"
    }

    val client = HttpClient(CIO) {
        install(ContentNegotiation) {
            json(
                Json {
                    ignoreUnknownKeys = true
                }
            )
        }
        install(UserAgent) {
            agent = "Microsoft-Delivery-Optimization/10.1"
        }
    }

    val schemas = client.getManifestSchemas()
    client.close()

    with(Terminal()) {
        println(yellow("Select mode:"))
        println("   ${optionBox('1')} ${blue("New Manifest or Package Version")}")
        println("   ${optionBox('2')} ${blue("Quick Update Package Version")}")
        println("   ${optionBox('3')} ${blue("Update Package Metadata")}")
        println("   ${optionBox('4')} ${blue("New Locale")}")
        println("   ${optionBox('5')} ${blue("Remove a manifest")}")
        println("   ${optionBox('Q')} ${red("Any key to quit")}")
        val selection = prompt(brightWhite("Selection"))
        println()
        when(selection) {
            "1" -> NewManifest(this, schemas).main() // TODO Handle nullability
            "2" -> TODO()
            "3" -> TODO()
            "4" -> TODO()
            "5" -> TODO()
            else -> exitProcess(0)
        }
    }
}

suspend fun HttpClient.getManifestSchemas(): List<Schema?> {
    return hashMapOf(
        Schemas.versionSchema to null as VersionSchema?,
        Schemas.installerSchema to null as InstallerSchema?
    ).also {
        it.forEach { entry ->
            try {
                it[entry.key] = get(entry.key).body()
            } catch (exception: NoTransformationFoundException) {
                val jsonString: String = get(entry.key).body()
                val json = Json { ignoreUnknownKeys = true }
                it[entry.key] = json.decodeFromJsonElement(json.parseToJsonElement(jsonString))
            }
        }
    }.map { it.value }
}
