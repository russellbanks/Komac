
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.ChangeToken
import commands.CommandOption
import commands.NewManifest
import commands.QuickUpdate
import commands.RemoveVersion
import input.Prompts
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import token.TokenStore

class Komac(private val args: Array<String>) : CliktCommand(invokeWithoutSubcommand = true), KoinComponent {
    override fun run() = runBlocking {
        get<TokenStore>()
        with(currentContext.terminal) {
            if (currentContext.invokedSubcommand == null) {
                println(
                    verticalLayout {
                        cell(colors.brightWhite("Select mode:"))
                        CommandOption.values().forEach { cell(optionCell(it)) }
                    }
                )
                val commandOption = prompt(
                    prompt = "Selection",
                    convert = { selection ->
                        val option = CommandOption.values().find {
                            it.key.toString().equals(other = selection, ignoreCase = true)
                        }
                        ConversionResult.Valid(option ?: CommandOption.Exit)
                    }
                )
                echo()
                executeSubcommand(commandOption)
            }
        }
    }

    private fun executeSubcommand(commandOption: CommandOption?) {
        when (commandOption) {
            CommandOption.NewManifest -> {
                this@Komac.registeredSubcommands().first { it::class == NewManifest::class }.main(args)
            }
            CommandOption.QuickUpdate -> {
                this@Komac.registeredSubcommands().first { it::class == QuickUpdate::class }.main(args)
            }
            CommandOption.RemoveVersion -> {
                this@Komac.registeredSubcommands().first { it::class == RemoveVersion::class }.main(args)
            }
            CommandOption.Token -> {
                this@Komac.registeredSubcommands().first { it::class == ChangeToken::class }.main(args)
            }
            else -> return
        }
    }

    private fun Terminal.optionCell(commandOption: CommandOption): String {
        val textColour = if (commandOption != CommandOption.Exit) colors.cyan else colors.brightRed
        return buildString {
            append(" ".repeat(Prompts.optionIndent))
            append(colors.cyan("["))
            append(colors.brightWhite(commandOption.key.toString()))
            append(colors.cyan("] "))
            append(textColour(commandOption.toString()))
        }
    }
}
