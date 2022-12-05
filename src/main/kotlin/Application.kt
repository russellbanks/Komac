import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.table.verticalLayout
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.TerminalInstance
import kotlin.system.exitProcess

class Application : KoinComponent {
    suspend fun main() {
        with(get<TerminalInstance>().terminal) {
            println(
                verticalLayout {
                    cell(yellow("Select mode:"))
                    Mode.values().forEach { mode ->
                        cell(option(mode, mode.key))
                    }
                }
            )
            val selection = prompt(brightWhite("Selection"), default = Mode.Exit.key.toString(), showDefault = false)
            println()
            when (selection?.lowercase()) {
                "1" -> NewManifest(this).main()
                "2" -> TODO()
                "3" -> TODO()
                "4" -> TODO()
                "5" -> TODO()
                else -> exitProcess(0)
            }
        }
    }

    private fun option(mode: Mode, key: Char, intent: Int = 3): String {
        val indent = " ".repeat(intent)
        val keyString = "${blue("[")}${brightWhite(key.toString())}${blue("]")}"
        val textColour = if (mode != Mode.Exit) blue else red
        return "$indent$keyString ${textColour(mode.toString())}"
    }
}
