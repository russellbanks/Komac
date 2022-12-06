import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.table.verticalLayout
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.TerminalInstance
import kotlin.system.exitProcess

class Application : KoinComponent {
    private val terminalInstance: TerminalInstance by inject()

    suspend fun main() {
        with(terminalInstance.terminal) {
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
                Mode.NewManifest.key.toString() -> NewManifest(this).main()
                Mode.QuickUpdate.key.toString() -> TODO()
                Mode.UpdateMetadata.key.toString() -> TODO()
                Mode.NewLocale.key.toString() -> TODO()
                Mode.RemoveManifest.key.toString() -> TODO()
                else -> exitProcess(0)
            }
        }
    }

    private fun option(mode: Mode, key: Char, intent: Int = 3): String {
        val textColour = if (mode != Mode.Exit) blue else red
        return buildString {
            append(" ".repeat(intent))
            append(blue("["))
            append(brightWhite(key.toString()))
            append(blue("] "))
            append(textColour(mode.toString()))
        }
    }
}
