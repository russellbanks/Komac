import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import io.ktor.client.HttpClient
import io.ktor.client.call.NoTransformationFoundException
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import schemas.ManifestVersionSchema
import schemas.Schemas
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
    }
    var manifestVersionSchema: ManifestVersionSchema?
    try {
        manifestVersionSchema = client.get(Schemas.manifestVersionSchema).body()
    } catch (exception: NoTransformationFoundException) {
        val mealsString: String = client.get(Schemas.manifestVersionSchema).body()
        val json = Json {
            ignoreUnknownKeys = true
        }
        manifestVersionSchema = json.decodeFromString(mealsString)
    }
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
            "1" -> NewManifest(this, manifestVersionSchema!!).run() // TODO Handle nullability
            "2" -> TODO()
            "3" -> TODO()
            "4" -> TODO()
            "5" -> TODO()
            else -> exitProcess(0)
        }
    }
}
