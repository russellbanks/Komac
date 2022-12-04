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
            println(verticalLayout {
                cell(yellow("Select mode:"))
                cell(option(Mode.NewManifest))
                cell(option(Mode.QuickUpdate))
                cell(option(Mode.UpdateMetadata))
                cell(option(Mode.NewLocale))
                cell(option(Mode.RemoveManifest))
                cell(option(Mode.Exit, key = "Q"))
            })
            val selection = prompt(brightWhite("Selection"), default = "Q", showDefault = false)
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

    private fun option(mode: Mode, intent: Int = 3, key: String = mode.ordinal.inc().toString()): String {
        val indent = " ".repeat(intent)
        val keyString = "${blue("[")}${brightWhite(key)}${blue("]")}"
        val textColour = if (mode != Mode.Exit) blue else red
        return "$indent${keyString} ${textColour(mode.toString())}"
    }
}
