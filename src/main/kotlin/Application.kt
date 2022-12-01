import com.github.ajalt.mordant.rendering.TextColors
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.TerminalInstance
import kotlin.system.exitProcess

class Application : KoinComponent {
    suspend fun main() {
        with(get<TerminalInstance>().terminal) {
            println(TextColors.yellow("Select mode:"))
            println("   ${optionBox('1')} ${TextColors.blue("New Manifest or Package Version")}")
            println("   ${optionBox('2')} ${TextColors.blue("Quick Update Package Version")}")
            println("   ${optionBox('3')} ${TextColors.blue("Update Package Metadata")}")
            println("   ${optionBox('4')} ${TextColors.blue("New Locale")}")
            println("   ${optionBox('5')} ${TextColors.blue("Remove a manifest")}")
            println("   ${optionBox('Q')} ${TextColors.red("Any key to quit")}")
            val selection = prompt(TextColors.brightWhite("Selection"))
            println()
            when (selection) {
                "1" -> NewManifest(this).main()
                "2" -> TODO()
                "3" -> TODO()
                "4" -> TODO()
                "5" -> TODO()
                else -> exitProcess(0)
            }
        }
    }
}

fun optionBox(char: Char): String {
    return "${TextColors.blue("[")}${TextColors.brightWhite(char.toString())}${TextColors.blue("]")}"
}
