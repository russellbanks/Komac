
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.table.verticalLayout
import input.Mode
import input.Prompts
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.TerminalInstance
import token.TokenStore
import kotlin.system.exitProcess

class Komac(private val args: Array<String>) : CliktCommand(invokeWithoutSubcommand = true), KoinComponent {
    override fun run(): Unit = runBlocking {
        with(get<TerminalInstance>().terminal) {
            get<TokenStore>().getToken(this)
            if (currentContext.invokedSubcommand == null) {
                println(
                    verticalLayout {
                        cell(brightYellow("Select mode:"))
                        cell("")
                        Mode.values().forEach { mode ->
                            cell(optionCell(mode, mode.key))
                        }
                        cell("")
                    }
                )
                val selection = prompt(
                    prompt = brightWhite("Selection"),
                    default = Mode.Exit.key.toString(),
                    showDefault = false
                )
                println()
                when (selection?.lowercase()) {
                    Mode.NewManifest.key.toString() -> {
                        this@Komac.registeredSubcommands().first { it::class == NewManifest::class }.main(args)
                    }
                    Mode.QuickUpdate.key.toString() -> {
                        this@Komac.registeredSubcommands().first { it::class == QuickUpdate::class }.main(args)
                    }
                    Mode.RemoveVersion.key.toString() -> {
                        this@Komac.registeredSubcommands().first { it::class == RemoveVersion::class }.main(args)
                    }
                    Mode.Token.key.toString() -> ChangeToken().run()
                    else -> exitProcess(0)
                }
            }
        }
    }

    private fun optionCell(mode: Mode, key: Char): String {
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
