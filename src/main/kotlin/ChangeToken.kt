import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import input.Polar
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.TerminalInstance
import token.TokenStore

class ChangeToken : KoinComponent {
    private val tokenStore: TokenStore by inject()
    fun run() {
        with(get<TerminalInstance>().terminal) {
            println(
                verticalLayout {
                    cell(brightYellow("Would you like to change the currently stored token?"))
                    Polar.values().forEach {
                        val textColour = if (it == Polar.Yes) brightGreen else brightWhite
                        cell(textColour("${" ".repeat(Prompts.optionIndent)} [${it.name.first()}] ${it.name}"))
                    }
                }
            )
            val answer = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = Polar.Yes.name.first(),
                convert = {
                    when (it.firstOrNull()?.lowercase()) {
                        Polar.Yes.name.first().lowercase() -> ConversionResult.Valid(Polar.Yes)
                        Polar.No.name.first().lowercase() -> ConversionResult.Valid(Polar.No)
                        else -> ConversionResult.Invalid("Invalid choice")
                    }
                }
            )
            if (answer == Polar.Yes) {
                tokenStore.promptForToken(this).also { tokenStore.putToken(it) }
                println(brightGreen("Token changed successfully"))
            } else {
                return
            }
        }
    }
}
