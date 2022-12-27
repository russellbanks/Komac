import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.optional
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.table.verticalLayout
import input.Mode
import input.Parameter
import input.Prompts
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.TerminalInstance
import kotlin.system.exitProcess

class Komac : CliktCommand(), KoinComponent {
    private val option: String? by argument().optional()

    override fun run(): Unit = runBlocking {
        val terminalInstance: TerminalInstance by inject()
        with(terminalInstance.terminal) {
            when(option?.lowercase()) {
                Parameter.New.name.lowercase() -> NewManifest(this).main()
                Parameter.Update.name.lowercase() -> QuickUpdate(this).main()
            }
            println(
                verticalLayout {
                    cell(brightYellow("Select mode:"))
                    cell("")
                    Mode.values().forEach { mode ->
                        cell(option(mode, mode.key))
                    }
                    cell("")
                }
            )
            val selection = prompt(brightWhite("Selection"), default = Mode.Exit.key.toString(), showDefault = false)
            println()
            when (selection?.lowercase()) {
                Mode.NewManifest.key.toString() -> NewManifest(this).main()
                Mode.QuickUpdate.key.toString() -> QuickUpdate(this).main()
                Mode.UpdateMetadata.key.toString() -> TODO()
                Mode.NewLocale.key.toString() -> TODO()
                Mode.RemoveManifest.key.toString() -> TODO()
                else -> exitProcess(0)
            }
        }
    }

    private fun option(mode: Mode, key: Char): String {
        val textColour = if (mode != Mode.Exit) cyan else brightRed
        return buildString {
            append(" ".repeat(Prompts.optionIndent))
            append(cyan("["))
            append(brightWhite(key.toString()))
            append(cyan("] "))
            append(textColour(mode.toString()))
        }
    }
}
